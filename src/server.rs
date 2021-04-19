use futures::channel::mpsc;
use futures::Stream;
use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use std::pin::Pin;
use tokio::time::{sleep, Duration};

use tonic::{transport::Server, Request, Response, Status};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }

    type LotsOfRepliesStream =
        Pin<Box<dyn Stream<Item = Result<HelloReply, Status>> + Send + Sync>>;

    async fn lots_of_replies(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<Self::LotsOfRepliesStream>, Status> {
        let (mut tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            let name = request.into_inner().name;
            for _ in 0..4 {
                match tx.try_send(Ok(HelloReply {
                    message: format!("hello {}", name),
                })) {
                    Err(err) => {
                        println!("err = {:?}", err);
                        break;
                    }
                    Ok(_) => {
                        sleep(Duration::from_millis(1000)).await;
                    }
                }
            }
        });

        // returning our reciever so that tonic can listen on reciever and send the response to client
        Ok(Response::new(Box::pin(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
