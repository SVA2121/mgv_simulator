mod mgv_lib;

fn main() {


    let offer = mgv_lib::Offer {
        side: mgv_lib::Side::Ask,
        price: 101,
        volume: 100,
        gasreq: 100,
    };
    println!("{:?}", offer);

    let offer2 = mgv_lib::Offer {
        side: mgv_lib::Side::Bid,
        price: 99,
        volume: 100,
        gasreq: 100,
    };
    println!("{:?}", offer2);

    let offer3 = mgv_lib::Offer {
        side: mgv_lib::Side::Ask,
        price: 102,
        volume: 100,
        gasreq: 100,
    };
    println!("{:?}", offer3);

    let offer4 = mgv_lib::Offer {
        side: mgv_lib::Side::Bid,
        price: 98,
        volume: 100,
        gasreq: 100,
    };
    println!("{:?}", offer4);

    println!("{}", offer > offer3);
    println!("{}", offer2 > offer4);

    let mut market = mgv_lib::Market::new();
    market.insert(offer);
    market.insert(offer2);
    market.insert(offer3);
    market.insert(offer4);
    println!("{:?}", market);
    println!("{:?}", market.best_bid());
    println!("{:?}", market.best_ask());

}
