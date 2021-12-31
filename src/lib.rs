pub mod shared;
pub mod tutorial;

use thrift::protocol::{TInputProtocol, TOutputProtocol};
use thrift_pool::{FromProtocol, ThriftConnection};
use tutorial::{CalculatorSyncClient, TCalculatorSyncClient};

impl<Ip: TInputProtocol, Op: TOutputProtocol> ThriftConnection for CalculatorSyncClient<Ip, Op> {
    type Error = thrift::Error;

    fn is_valid(&mut self) -> Result<(), Self::Error> {
        self.ping()
    }

    fn has_broken(&mut self) -> bool {
        self.ping().is_err()
    }
}

impl<Ip: TInputProtocol, Op: TOutputProtocol> FromProtocol for CalculatorSyncClient<Ip, Op> {
    type InputProtocol = Ip;

    type OutputProtocol = Op;

    fn from_protocol(
        input_protocol: Self::InputProtocol,
        output_protocol: Self::OutputProtocol,
    ) -> Self {
        Self::new(input_protocol, output_protocol)
    }
}
