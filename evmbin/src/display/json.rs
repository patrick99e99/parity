// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! JSON VM output.

use ethcore::trace;
use std::collections::HashMap;
use util::{U256, H256, ToPretty};

use display;
use vm;

/// JSON formatting informant.
#[derive(Default)]
pub struct Informant {
	depth: usize,
	pc: usize,
	instruction: u8,
	gas_cost: U256,
	stack: Vec<U256>,
	memory: Vec<u8>,
	storage: HashMap<H256, H256>,
}

impl Informant {
	fn memory(&self) -> String {
		format!("\"0x{}\"", self.memory.to_hex())
	}

	fn stack(&self) -> String {
		let items = self.stack.iter().map(display::u256_as_str).collect::<Vec<_>>();
		format!("[{}]", items.join(","))
	}

	fn storage(&self) -> String {
		let vals = self.storage.iter()
			.map(|(k, v)| format!("\"0x{:?}\": \"0x{:?}\"", k, v))
			.collect::<Vec<_>>();
		format!("{{{}}}", vals.join(","))
	}
}

impl vm::Informant for Informant {
	fn finish(&mut self, result: Result<vm::Success, vm::Failure>) {
		match result {
			Ok(success) => println!(
				"{{\"output\":\"0x{output}\",\"gasUsed\":\"{gas:x}\",\"time\":\"{time}\"}}",
				output = success.output.to_hex(),
				gas = success.gas_used,
				time = display::format_time(&success.time),
			),
			Err(failure) => println!(
				"{{\"error\":\"{error}\",\"time\":\"{time}\"}}",
				error = failure.error,
				time = display::format_time(&failure.time),
			),
		}
	}
}

impl trace::VMTracer for Informant {
	fn trace_prepare_execute(&mut self, pc: usize, instruction: u8, stack_pop: usize, gas_cost: &U256) -> bool {
		self.pc = pc;
		self.instruction = instruction;
		self.gas_cost = *gas_cost;
		let len = self.stack.len();
		self.stack.truncate(len - stack_pop);
		true
	}

	fn trace_executed(&mut self, gas_used: U256, stack_push: &[U256], mem_diff: Option<(usize, &[u8])>, store_diff: Option<(U256, U256)>) {
		self.stack.extend_from_slice(stack_push);

		if let Some((pos, data)) = mem_diff {
			self.memory[pos..pos + data.len()].copy_from_slice(data);
		}

		if let Some((pos, val)) = store_diff {
			self.storage.insert(pos.into(), val.into());
		}

		println!(
			"{{\"pc\":{pc},\"op\":{op},\"gas\":{gas},\"gasCost\":{gas_cost},\"memory\":{memory},\"stack\":{stack},\"storage\":{storage},\"depth\":{depth}}}",
			pc = self.pc,
			op = self.instruction,
			gas = display::u256_as_str(&gas_used),
			gas_cost = display::u256_as_str(&self.gas_cost),
			memory = self.memory(),
			stack = self.stack(),
			storage = self.storage(),
			depth = self.depth,
		);
	}

	fn prepare_subtrace(&self, _code: &[u8]) -> Self where Self: Sized {
		let mut vm = Informant::default();
		vm.depth = self.depth + 1;
		vm
	}

	fn done_subtrace(&mut self, _sub: Self) where Self: Sized {}
	fn drain(self) -> Option<trace::VMTrace> { None }
}
