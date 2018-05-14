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
usually called `KRPC.<some service>.json`. For instance, the most important one
is `KRPC.SpaceCenter.json` and contains the definition of the main RPCs you
will want to use. Put these files in some directory within your project.

[krpc-mars-terraformer]: https://github.com/Cahu/krpc-mars-terraformer

    $ cd betterjeb
    $ mkdir services
    $ cp ~/games/KSP/game/GameData/kRPC/KRPC.*.json services/

Since these service files are likely to change often, the point of this step is
to be able to use a new version of the kRPC mod without updating `krpc-mars`.

We now need to write a `build.rs` script to instruct cargo to generate the
files before building our project :

    // FILE: build.rs
    extern crate glob;
    extern crate krpc_mars_terraform;

    fn main() {
        // Tell cargo to re-run this script only when json files in services/
        // have changed. You can choose to omit this step if you want to
        // re-generate services every time.
        for path in glob::glob("services/*.json").unwrap().filter_map(Result::ok) {
            println!("cargo:rerun-if-changed={}", path.display());
        }

        krpc_mars_terraform::run("services/", "src/")
            .expect("Could not terraform Mars :(");
    }

List dependencies in the `Cargo.toml` file :

    [package]
    name = "betterjeb"
    version = "0.1.0"
    authors = ["Jeb"]

    [dependencies]
    krpc_mars = { git = ... }

    [build-dependencies]
    glob = "*"
    krpc_mars_terraform = { git = ... }

The last step is to list all generated services in `src/lib.rs` :

    // FILE: lib.rs
    pub extern crate krpc_mars; // don't forget this
    pub mod drawing;
    pub mod infernal_robotics;
    pub mod kerbal_alarm_clock;
    pub mod remote_tech;
    pub mod space_center;
    pub mod ui;

We're good. Let's compile !

    $ cargo build

Let's have some documentation too !

    $ cargo doc --open

Everything is ready. Here is a `main.rs` to get you started :

    // FILE: main.rs
    extern crate betterjeb;
    use betterjeb::*;

    use std::thread;
    use std::time::Duration;

    fn main() {
        let client = krpc_mars::RPCClient::connect("Example", "127.0.0.1:50000").unwrap();

        // Say something nice
        let ui = ui::UI::new(client.clone());
        ui.message("It works!!".to_string(), 5f32, ui::MessagePosition::BottomCenter)
            .expect("Wait ... in fact it doesn't work ...");

        // Tour of the solar system ...
        let space_center = space_center::SpaceCenter::new(client.clone());
        let bodies = space_center.get_bodies().unwrap();
        let camera = space_center.get_camera().unwrap();
        camera.set_mode(space_center::CameraMode::Map).unwrap();
        for (_body_name, body_object) in &bodies {
            camera.set_focussed_body(body_object).unwrap();
            thread::sleep(Duration::from_millis(5000));
        }
        camera.set_mode(space_center::CameraMode::Automatic).unwrap();
    }
