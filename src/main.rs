use futures_lite::stream::StreamExt;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties,
};
use std::env;
use std::error::Error;

const QUEUE_NAME: &str = "example";

async fn declare_queue(channel: &Channel)  -> Result<(), Box<dyn Error>> {
    channel
        .queue_declare(
            QUEUE_NAME,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    Ok(())
}

async fn emit(channel: Channel) -> Result<(), Box<dyn Error>> {
    declare_queue(&channel)
        .await
        .expect("queue declaration");

    channel
        .confirm_select(ConfirmSelectOptions::default())
        .await
        .expect("select confirmation");

    
    let payload = b"Hello world!";
    channel
        .basic_publish(
            "",
            QUEUE_NAME,
            BasicPublishOptions::default(),
            payload,
            BasicProperties::default(),
        )
        .await
        .expect("basic_publish")
        .await // Wait for this specific ack/nack
        .expect("publisher-confirms");


    println!("Message sent to {}: Hello world!", QUEUE_NAME);
    Ok(())
}

async fn receive(channel: Channel) -> Result<(), Box<dyn Error>> {
    declare_queue(&channel)
        .await
        .expect("queue declaration");

    let mut consumer = channel
        .basic_consume(
            QUEUE_NAME,
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("consumer declaration");

    println!("[LOOP] Click Ctrl + C to kill this process...");
    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");
        let message = String::from_utf8(delivery.data.clone()).expect("get message from utf-8");
        println!("received message: {}", message);

        delivery.ack(BasicAckOptions::default()).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const OPTIONS: &[&str] = &["-e", "-r"];
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || !OPTIONS.iter().any(|option| option == &args[1]) {
        println!("USAGE:\n\trabbit-test <option>\n");
        println!("OPTIONS:\n\t-e\temit log message\n\t-r\treceive message\n");
        return Ok(());
    }

    const ADDR: &str = "amqp://127.0.0.1:5672/%2f";
    let connection = Connection::connect(ADDR, ConnectionProperties::default())
        .await
        .expect("connection error");
    let channel = connection.create_channel().await.expect("channel creation");

    if args[1] == OPTIONS[0] {
        emit(channel).await?;
    } else if args[1] == OPTIONS[1] {
        receive(channel).await?;
    }

    Ok(())
}
