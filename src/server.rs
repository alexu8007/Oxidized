use crate::{Error, Request, Response, Result, Service};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn as hyper_service_fn;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;

pub struct Server<S> {
    service: S,
    addr: SocketAddr,
    tls_config: Option<TlsConfig>,
}

struct TlsConfig {
    cert_path: PathBuf,
    key_path: PathBuf,
}

impl<S> Server<S> {
    pub fn new(service: S, addr: SocketAddr) -> Self {
        Self {
            service,
            addr,
            tls_config: None,
        }
    }

    pub fn tls(mut self, cert_path: impl Into<PathBuf>, key_path: impl Into<PathBuf>) -> Self {
        self.tls_config = Some(TlsConfig {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
        });
        self
    }

    pub async fn run(self) -> Result<()>
    where
        S: Service<Request, Response = Response, Error = Error> + Clone + Send + Sync + 'static,
        S::Future: Send,
    {
        let service = Arc::new(self.service);

        if let Some(tls_config) = self.tls_config {
            let acceptor = tls_config.acceptor()?;
            let listener = TcpListener::bind(self.addr).await?;

            loop {
                let (stream, _) = listener.accept().await?;
                let service = service.clone();
                let acceptor = acceptor.clone();

                tokio::spawn(async move {
                    match acceptor.accept(stream).await {
                        Ok(stream) => {
                            let io = TokioIo::new(stream);
                            let hyper_service =
                                hyper_service_fn(move |req: hyper::Request<Incoming>| {
                                    let service = service.clone();
                                    async move {
                                        let res = service.call(Request::from_hyper(req)).await;
                                        match res {
                                            Ok(res) => Ok::<_, hyper::Error>(res.into_hyper()),
                                            Err(_) => {
                                                let mut res =
                                                    Response::new("Not Found").into_hyper();
                                                *res.status_mut() = StatusCode::NOT_FOUND;
                                                Ok(res)
                                            }
                                        }
                                    }
                                });

                            if let Err(err) = http1::Builder::new()
                                .serve_connection(io, hyper_service)
                                .await
                            {
                                eprintln!("server error: {}", err);
                            }
                        }
                        Err(err) => {
                            eprintln!("tls error: {}", err);
                        }
                    }
                });
            }
        } else {
            let listener = TcpListener::bind(self.addr).await?;
            loop {
                let (stream, _) = listener.accept().await?;
                let io = TokioIo::new(stream);
                let service = service.clone();

                tokio::spawn(async move {
                    let hyper_service = hyper_service_fn(move |req: hyper::Request<Incoming>| {
                        let service = service.clone();
                        async move {
                            let res = service.call(Request::from_hyper(req)).await;
                            match res {
                                Ok(res) => Ok::<_, hyper::Error>(res.into_hyper()),
                                Err(_) => {
                                    let mut res = Response::new("Not Found").into_hyper();
                                    *res.status_mut() = StatusCode::NOT_FOUND;
                                    Ok(res)
                                }
                            }
                        }
                    });

                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, hyper_service)
                        .await
                    {
                        eprintln!("server error: {}", err);
                    }
                });
            }
        }
    }
}

impl TlsConfig {
    fn acceptor(&self) -> Result<Arc<TlsAcceptor>> {
        let certs = load_certs(&self.cert_path)?;
        let key = load_key(&self.key_path)?;

        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        Ok(Arc::new(TlsAcceptor::from(Arc::new(config))))
    }
}

fn load_certs(path: &Path) -> std::io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect())
}

fn load_key(path: &Path) -> std::io::Result<PrivateKey> {
    pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid key"))
        .and_then(|mut keys| {
            keys.pop()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no key found"))
        })
        .map(PrivateKey)
} 