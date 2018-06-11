# krpc-mars

This is client code for [kRPC] -- *Kerbal Remote Procedure Call*, a mod for
Kerbal Space Program -- written in Rust. Mars is red because of rust on its
surface, hence the name of this library.

[kRPC]: https://github.com/krpc/krpc

## How to use

First, create your project

    $ cargo new --bin betterjeb
          Created binary (application) `betterjeb` project

You will need the `.json` service files bundled with the kRPC mod and the
[krpc-mars-terraformer] library to generate rust code from them. These files are
usually called `KRPC.<some service>.json` (there is also a file called
`KRPC.json`, ignore it). For instance, the most important one is
`KRPC.SpaceCenter.json` and contains the definition of the main RPCs you will
want to use. Put these files in some directory within your project.

[krpc-mars-terraformer]: https://github.com/Cahu/krpc-mars-terraformer

    $ cd betterjeb
    $ mkdir services
    $ cp ~/games/KSP/game/GameData/kRPC/KRPC.*.json services/

Since these service files are likely to change often, the point of this step is
to be able to use a new version of the kRPC mod without updating `krpc-mars`.

We now need to write a `build.rs` script to instruct cargo to generate the
files before building our project :

```rust
// FILE: build.rs
extern crate glob;
extern crate krpc_mars_terraformer;

fn main() {
	// Tell cargo to re-run this script only when json files in services/
	// have changed. You can choose to omit this step if you want to
	// re-generate services every time.
	for path in glob::glob("services/*.json").unwrap().filter_map(Result::ok) {
		println!("cargo:rerun-if-changed={}", path.display());
	}

	krpc_mars_terraformer::run("services/", "src/")
		.expect("Could not terraform Mars :(");
}
```

List dependencies in the `Cargo.toml` file :

```toml
[package]
name = "betterjeb"
version = "0.1.0"
authors = ["Jeb"]

[dependencies]
krpc_mars = { git = ... }

[build-dependencies]
glob = "*"
krpc_mars_terraformer = { git = ... }
```

The last step is to list all generated services in `src/lib.rs` :

```rust
// FILE: lib.rs
extern crate krpc_mars; // don't forget this
pub mod drawing;
pub mod infernal_robotics;
pub mod kerbal_alarm_clock;
pub mod remote_tech;
pub mod space_center;
pub mod ui;
```

We're good. Let's compile !

    $ cargo build

Let's have some documentation too !

    $ cargo doc --open


## Examples

### Simple RPC

Here is a basic example using RPCs from the SpaceCenter service:

```rust
extern crate betterjeb;
use betterjeb::*;

extern crate krpc_mars;

extern crate failure;

fn main() -> Result<(), failure::Error> {
    let client = krpc_mars::RPCClient::connect("Example", "127.0.0.1:50000")?;

    let vessel = client.mk_call(&space_center::get_active_vessel())?;
    println!("Active vessel: {:?}", vessel);

    let crew = client.mk_call(&vessel.get_crew())?;
    println!("Crew: {:?}", crew);

    Ok(())
}
```

When you run this program you should see something like this:

```
Active vessel: Vessel(1)
Crew: [CrewMember(2)]
```

These numbers are ids created by the kRPC server.

### Batches

You can also group RPCs in batches, meaning multiple calls will be grouped in a
single packet. For instance:

```rust
let client = krpc_mars::RPCClient::connect("Example", "127.0.0.1:50000")?;

let (vessel, time) = batch_call!(&client, (
    &space_center::get_active_vessel(),
    &space_center::get_ut(),
))?;

let time = time?;
let vessel = vessel?;

println!("Current time: {}, Vessel: {:?}", time, vessel);

let (crew, _, _) = batch_call!(&client, (
    &vessel.get_crew(),
    &vessel.set_type(space_center::VesselType::Probe),
    &ui::message("Vessel type set to 'Probe'!".to_string(), 5f32, ui::MessagePosition::TopCenter),
))?;

println!("Crew: {:?}", crew?);
```

If you don't want to unwrap all return values manually, then you can use the
`batch_call_unwrap!` macro instead of `batch_call!`.

### Using streams

Streams are easy to setup, just use `to_stream()` on the regular function. You
will get a `StreamHandle` which you can use later to retrieve the result of
this particular RPC from a `StreamUpdate` you obtained by calling
`recv_update()` on the stream client.

```rust
let client = krpc_mars::RPCClient::connect("Example", "127.0.0.1:50000")?;
let stream_client = krpc_mars::StreamClient::connect(&client, "127.0.0.1:50001")?;

let ut_stream_handle = client.mk_call(&space_center::get_ut().to_stream())?;

loop {
    let update = stream_client.recv_update()?;
    let ut_result = update.get_result(&ut_stream_handle)?;
    println!("ut: {}", ut_result);
}
```

Of course, you can also create a stream using batches :

```rust
let client = krpc_mars::RPCClient::connect("Example", "127.0.0.1:50000")?;
let stream_client = krpc_mars::StreamClient::connect(&client, "127.0.0.1:50001")?;

let (vessel, ut_stream_handle) = batch_call_unwrap!(&client, (
    &space_center::get_active_vessel(),
    &space_center::get_ut().to_stream(),
))?;

println!("Current vessel: {:?}", vessel);

loop {
    let update = stream_client.recv_update()?;
    let ut_result = update.get_result(&ut_stream_handle)?;
    println!("ut: {}", ut_result);
}
```

To remove a stream, use the `remove()` method of the stream handle which will
generate the appropriate request:

```
client.mk_call(&ut_stream_handle.remove())?;
```
