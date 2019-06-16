use futures::{Future, Sink, Stream};
use vitruvia::{
    executor,
    network::mesh::{Channel, Peer},
};

#[macro_use]
extern crate stdweb;

fn main() {
    executor::run(
        Peer::new()
            .join(Peer::new())
            .map_err(|err| {
                eprintln!("{:?}", err);
                ()
            })
            .and_then(|((peer, negotiation), (peer0, negotiation0))| {
                let (i, o) = negotiation.split();
                let (i0, o0) = negotiation0.split();

                let mut peer0 = peer0;
                let channel = peer0.data_channel();

                executor::spawn(channel.then(|channel| {
                    channel
                        .unwrap()
                        .send(vec![0])
                        .map_err(|err| {
                            eprintln!("{:?}", err);
                            ()
                        })
                        .and_then(|_| Ok(()))
                }));

                peer.for_each(|channel| {
                    let channel = match channel {
                        Channel::DataChannel(data_channel) => data_channel,
                        _ => {
                            panic!("not data channel");
                        }
                    };
                    executor::spawn(
                        channel
                            .for_each(|message| {
                                console!(log, format!("got message: {:?}", message));
                                Ok(())
                            })
                            .map_err(|err| {
                                eprintln!("{:?}", err);
                                ()
                            }),
                    );
                    Ok(())
                })
                .join(o.forward(i0).join(o0.forward(i)))
                .map_err(|err| {
                    eprintln!("{:?}", err);
                    ()
                })
                .and_then(|(_, _)| Ok(()))
            }),
    );
}
