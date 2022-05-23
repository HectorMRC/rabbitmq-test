use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, ExchangeDeclareOptions, ExchangeType, FieldTable,
    Publish, QueueDeclareOptions, Result, Channel
};
use std::env;

fn emit(channel: Channel) -> Result<()> {
    // Declare the fanout exchange we will publish to.
    let exchange = channel.exchange_declare(
        ExchangeType::Fanout,
        "logs",
        ExchangeDeclareOptions::default(),
    )?;

    // Publish a message to the logs queue.
    let mut message = env::args().skip(1).collect::<Vec<_>>().join(" ");
    if message.is_empty() {
        message = "info: Hello world!".to_string();
    }

    exchange.publish(Publish::new(message.as_bytes(), ""))?;
    println!("Sent [{}]", message);

    Ok(())
}


fn receive(channel: Channel) -> Result<()> {
    // Declare the fanout exchange we will bind to.
    let exchange = channel.exchange_declare(
        ExchangeType::Fanout,
        "logs",
        ExchangeDeclareOptions::default(),
    )?;

    // Declare the exclusive, server-named queue we will use to consume.
    let queue = channel.queue_declare(
        "",
        QueueDeclareOptions {
            exclusive: true,
            ..QueueDeclareOptions::default()
        },
    )?;
    println!("created exclusive queue {}", queue.name());

    // Bind our queue to the logs exchange.
    queue.bind(&exchange, "", FieldTable::new())?;

    // Start a consumer. Use no_ack: true so the server doesn't wait for us to ack
    // the messages it sends us.
    let consumer = queue.consume(ConsumerOptions {
        no_ack: true,
        ..ConsumerOptions::default()
    })?;
    println!("Waiting for logs. Press Ctrl-C to exit.");

    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                println!("({:>3}) {}", i, body);
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    const OPTIONS: &[&str] = &["-e", "-r"];
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || !OPTIONS.iter().any(|option| option == &args[1]) {
        println!("USAGE:\n\trabbit-test <option>\n");
        println!("OPTIONS:\n\t-e\temit log message\n\t-r\treceive message\n");
        return Ok(())
    }

    env_logger::init();

    const ADDR: &str = "amqp://127.0.0.1:5672/%2f";
    let mut connection = Connection::insecure_open(ADDR)?;
    let channel = connection.open_channel(None)?;

    if args[1] == OPTIONS[0] {
        emit(channel)?;
    } else if args[1] == OPTIONS[1] {
        receive(channel)?;
    }

    connection.close()
}
