use r2d2::Pool;
use thrift::protocol::{TCompactInputProtocol, TCompactOutputProtocol};
use thrift::transport::{
    ReadHalf, TFramedReadTransport, TFramedWriteTransport, TTcpChannel, WriteHalf,
};

use thrift_pool::MakeThriftConnectionFromAddrs;
use thrift_pool_tutorial::shared::TSharedServiceSyncClient;
use thrift_pool_tutorial::tutorial::{
    CalculatorSyncClient, Operation, TCalculatorSyncClient, Work,
};

type Client = CalculatorSyncClient<
    TCompactInputProtocol<TFramedReadTransport<ReadHalf<TTcpChannel>>>,
    TCompactOutputProtocol<TFramedWriteTransport<WriteHalf<TTcpChannel>>>,
>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let remote_address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:9090".to_string());
    let manager =
        MakeThriftConnectionFromAddrs::<Client, _>::new(remote_address).into_connection_manager();
    let pool = Pool::builder().build(manager)?;
    let mut client = pool.get()?;

    // alright!
    // let's start making some calls

    // let's start with a ping; the server should respond
    println!("ping!");
    client.ping()?;

    // simple add
    println!("add");
    let res = client.add(1, 2)?;
    println!("added 1, 2 and got {}", res);

    let logid = 32;

    // let's do...a multiply!
    let res = client.calculate(logid, Work::new(7, 8, Operation::MULTIPLY, None))?;
    println!("multiplied 7 and 8 and got {}", res);

    // let's get the log for it
    let res = client.get_struct(logid /* 32 */)?;
    println!("got log {:?} for operation {}", res, logid);

    // ok - let's be bad :(
    // do a divide by 0
    // logid doesn't matter; won't be recorded
    let res = client.calculate(77, Work::new(2, 0, Operation::DIVIDE, "we bad".to_owned()));

    // we should have gotten an exception back
    match res {
        Ok(v) => panic!("should not have succeeded with result {}", v),
        Err(e) => println!("divide by zero failed with error {:?}", e),
    }

    // let's do a one-way call
    println!("zip");
    client.zip()?;

    // and then close out with a final ping
    println!("ping!");
    client.ping()?;

    Ok(())
}
