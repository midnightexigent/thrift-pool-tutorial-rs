use std::collections::HashMap;
use std::convert::{From, Into};
use std::default::Default;
use std::sync::Mutex;

use thrift::protocol::{TCompactInputProtocolFactory, TCompactOutputProtocolFactory};
use thrift::server::TServer;
use thrift::transport::{TFramedReadTransportFactory, TFramedWriteTransportFactory};

use thrift_pool_tutorial::shared::{SharedServiceSyncHandler, SharedStruct};
use thrift_pool_tutorial::tutorial::{CalculatorSyncHandler, CalculatorSyncProcessor};
use thrift_pool_tutorial::tutorial::{InvalidOperation, Operation, Work};

fn main() -> thrift::Result<()> {
    let listen_address = format!(
        "127.0.0.1:{}",
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "9090".to_string()),
    );

    println!("binding to {}", listen_address);

    let i_tran_fact = TFramedReadTransportFactory::new();
    let i_prot_fact = TCompactInputProtocolFactory::new();

    let o_tran_fact = TFramedWriteTransportFactory::new();
    let o_prot_fact = TCompactOutputProtocolFactory::new();

    // demux incoming messages
    let processor = CalculatorSyncProcessor::new(CalculatorServer {
        ..Default::default()
    });

    // create the server and start listening
    let mut server = TServer::new(
        i_tran_fact,
        i_prot_fact,
        o_tran_fact,
        o_prot_fact,
        processor,
        10,
    );

    server.listen(&listen_address)
}

/// Handles incoming Calculator service calls.
struct CalculatorServer {
    log: Mutex<HashMap<i32, SharedStruct>>,
}

impl Default for CalculatorServer {
    fn default() -> CalculatorServer {
        CalculatorServer {
            log: Mutex::new(HashMap::new()),
        }
    }
}

// since Calculator extends SharedService we have to implement the
// handler for both traits.
//

// SharedService handler
impl SharedServiceSyncHandler for CalculatorServer {
    fn handle_get_struct(&self, key: i32) -> thrift::Result<SharedStruct> {
        let log = self.log.lock().unwrap();
        log.get(&key)
            .cloned()
            .ok_or_else(|| format!("could not find log for key {}", key).into())
    }
}

// Calculator handler
impl CalculatorSyncHandler for CalculatorServer {
    fn handle_ping(&self) -> thrift::Result<()> {
        println!("pong!");
        Ok(())
    }

    fn handle_add(&self, num1: i32, num2: i32) -> thrift::Result<i32> {
        println!("handling add: n1:{} n2:{}", num1, num2);
        Ok(num1 + num2)
    }

    fn handle_calculate(&self, logid: i32, w: Work) -> thrift::Result<i32> {
        println!("handling calculate: l:{}, w:{:?}", logid, w);

        let res = if let Some(ref op) = w.op {
            if w.num1.is_none() || w.num2.is_none() {
                Err(InvalidOperation {
                    what_op: Some(op.into()),
                    why: Some("no operands specified".to_owned()),
                })
            } else {
                // so that I don't have to call unwrap() multiple times below
                let num1 = w.num1.as_ref().expect("operands checked");
                let num2 = w.num2.as_ref().expect("operands checked");

                match *op {
                    Operation::ADD => Ok(num1 + num2),
                    Operation::SUBTRACT => Ok(num1 - num2),
                    Operation::MULTIPLY => Ok(num1 * num2),
                    Operation::DIVIDE => {
                        if *num2 == 0 {
                            Err(InvalidOperation {
                                what_op: Some(op.into()),
                                why: Some("divide by 0".to_owned()),
                            })
                        } else {
                            Ok(num1 / num2)
                        }
                    }
                    _ => {
                        let op_val: i32 = op.into();
                        Err(InvalidOperation {
                            what_op: Some(op_val),
                            why: Some(format!("unsupported operation type '{}'", op_val)),
                        })
                    }
                }
            }
        } else {
            Err(InvalidOperation::new(
                None,
                "no operation specified".to_owned(),
            ))
        };

        // if the operation was successful log it
        if let Ok(ref v) = res {
            let mut log = self.log.lock().unwrap();
            log.insert(logid, SharedStruct::new(logid, format!("{}", v)));
        }

        // the try! macro automatically maps errors
        // but, since we aren't using that here we have to map errors manually
        //
        // exception structs defined in the IDL have an auto-generated
        // impl of From::from
        res.map_err(From::from)
    }

    fn handle_zip(&self) -> thrift::Result<()> {
        println!("handling zip");
        Ok(())
    }
}
