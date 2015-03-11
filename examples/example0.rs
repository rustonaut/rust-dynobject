extern crate dynobject;

use dynobject::DynObject;

//for simplicity use &'static str
type Key = &'static str;

//with intorior mutablility
//if you do not want this just change
//`fn run(&self)...` to `fn run(&mut self)...`
struct Processor {
	shared_data: DynObject<Key>,
	//there are will soon be unboxed closures witch could be used here insted of Box<Fn...>
	runner: Box<Fn(&DynObject<Key>) -> bool>
}

impl Processor {
	fn run(&self) -> bool {
		(self.runner)(&self.shared_data)
	}
}

fn run_to_end(pr: &Processor) {
	while pr.run() { 
		println!("running");
	}
}

fn setup_data(outer_obj: &DynObject<Key>) {
	let mut obj = outer_obj.aquire();
	obj.create_property("counter1", Box::new(0u32));
	obj.create_property("counter2", Box::new(1u32));
	obj.create_property("limit", Box::new(4u32));
}

fn main() {
	let obj = DynObject::<Key>::new();
	setup_data( &obj );
	let p1 = Processor {
		shared_data: obj.clone(),
		runner: Box::new( |data: &DynObject<Key>| -> bool {
			let mut obj = data.aquire();
			let value = *obj["counter1"].as_ref::<u32>().unwrap() + 1;
			*obj["counter1"].as_mut::<u32>().unwrap() = value;
			println!("reached {}", value);
			obj["limit"].as_ref::<u32>().unwrap() >= &value
		} )
	};
	let p2 = Processor {
		shared_data: obj.clone(),
		runner: Box::new( |data: &DynObject<Key>| -> bool {
			let mut obj = data.aquire();
			*obj["counter2"].as_mut::<u32>().unwrap() += 2;
            {
                let ref_2_counter1 = obj["counter1"].as_mut::<u32>().unwrap();
                *ref_2_counter1 -= 1;
                *ref_2_counter1 > 0
            }
		} )
	};
	
	run_to_end( &p1 );
	run_to_end( &p2 );

    let accessor = obj.aquire();
    let c1 = accessor["counter1"].as_ref::<u32>().unwrap();
    let c2 = accessor["counter2"].as_ref::<u32>().unwrap();
    let limit = accessor["limit"].as_ref::<u32>().unwrap();
	println!( "c1: {}, c2: {}, limit: {}", c1, c2, limit );
}	
