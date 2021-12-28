mod shared;
mod tutorial;

use shared::TSharedServiceSyncClient;
use thrift::{
    protocol::{TCompactInputProtocol, TCompactOutputProtocol, TInputProtocol, TOutputProtocol},
    transport::{ReadHalf, TFramedReadTransport, TFramedWriteTransport, TTcpChannel, WriteHalf},
};
use tutorial::{CalculatorSyncClient, Operation, TCalculatorSyncClient, Work};

use thrift_pool::{
    r2d2::Pool, FromIoProtocol, HasBroken, IsValid, MakeCompactProtocol, MakeFramedTransport,
    ThriftConnectionManager,
};

impl<IP: TInputProtocol, OP: TOutputProtocol> FromIoProtocol for CalculatorSyncClient<IP, OP> {
    type InputProtocol = IP;

    type OutputProtocol = OP;

    fn from_io_protocol(
        input_protocol: Self::InputProtocol,
        output_protocol: Self::OutputProtocol,
    ) -> Self {
        CalculatorSyncClient::new(input_protocol, output_protocol)
    }
}
impl<IP: TInputProtocol, OP: TOutputProtocol> IsValid for CalculatorSyncClient<IP, OP> {
    fn is_valid(&mut self) -> Result<(), thrift::Error> {
        self.ping()
    }
}
impl<IP: TInputProtocol, OP: TOutputProtocol> HasBroken for CalculatorSyncClient<IP, OP> {
    fn has_broken(&mut self) -> bool {
        self.ping().is_err()
    }
}

type ClientInputProtocol = TCompactInputProtocol<TFramedReadTransport<ReadHalf<TTcpChannel>>>;
type ClientOutputProtocol = TCompactOutputProtocol<TFramedWriteTransport<WriteHalf<TTcpChannel>>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ThriftConnectionManager::<
        CalculatorSyncClient<ClientInputProtocol, ClientOutputProtocol>,
        _,
        _,
        _,
        _,
        _,
    >::new(
        "localhost:9090",
        MakeCompactProtocol::new(),
        MakeCompactProtocol::new(),
        MakeFramedTransport::new(),
        MakeFramedTransport::new(),
    );
    let pool = Pool::builder().build(manager)?;
    let mut conn = pool.get()?;

    println!("add");
    let res = conn.add(1, 2)?;
    println!("added 1, 2 and got {}", res);

    let logid = 32;

    // let's do...a multiply!
    let res = conn.calculate(logid, Work::new(7, 8, Operation::MULTIPLY, None))?;
    println!("multiplied 7 and 8 and got {}", res);

    // let's get the log for it
    let res = conn.get_struct(logid /* 32 */)?;
    println!("got log {:?} for operation {}", res, logid);

    // ok - let's be bad :(
    // do a divide by 0
    // logid doesn't matter; won't be recorded
    let res = conn.calculate(77, Work::new(2, 0, Operation::DIVIDE, "we bad".to_owned()));

    // we should have gotten an exception back
    match res {
        Ok(v) => panic!("should not have succeeded with result {}", v),
        Err(e) => println!("divide by zero failed with error {:?}", e),
    }

    // let's do a one-way call
    println!("zip");
    conn.zip()?;

    // and then close out with a final ping
    println!("ping!");
    conn.ping()?;
    Ok(())
}
