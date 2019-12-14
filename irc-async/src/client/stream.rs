use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_tls::TlsStream;

pub enum ClientStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl AsyncRead for ClientStream {
    fn poll_read(
        self: Pin<&mut Self>,
        context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.get_mut() {
            ClientStream::Plain(stream) => TcpStream::poll_read(Pin::new(stream), context, buf),
            ClientStream::Tls(stream) => {
                TlsStream::<TcpStream>::poll_read(Pin::new(stream), context, buf)
            }
        }
    }
}

impl AsyncWrite for ClientStream {
    fn poll_write(
        self: Pin<&mut Self>,
        context: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.get_mut() {
            ClientStream::Plain(stream) => TcpStream::poll_write(Pin::new(stream), context, buf),
            ClientStream::Tls(stream) => {
                TlsStream::<TcpStream>::poll_write(Pin::new(stream), context, buf)
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            ClientStream::Plain(stream) => TcpStream::poll_flush(Pin::new(stream), context),
            ClientStream::Tls(stream) => {
                TlsStream::<TcpStream>::poll_flush(Pin::new(stream), context)
            }
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            ClientStream::Plain(stream) => TcpStream::poll_shutdown(Pin::new(stream), context),
            ClientStream::Tls(stream) => {
                TlsStream::<TcpStream>::poll_shutdown(Pin::new(stream), context)
            }
        }
    }
}
