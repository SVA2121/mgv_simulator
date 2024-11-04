# Mangrove Simulator

This is a simulator for Mangrove, written in Rust.

## Goal
Have a fast reliable simulator for Mangrove, that can be used for testing and simulation of Mangrove strategies. 

It is to be improved in accuracy, speed and memory usage. It also needs to include Mangrove's desgin to be able to simulate as close as possible to the real on=chain behavior.

##

v0.1.0 is a simple simulator that can read and write offers, and match limit orders. It's offers need to be thinked as Mangroves orders i.e post-hook logic i.e the reposting is done in the same transaction as the market order.

It's initial application will be a simple Kandel strategy.
We need to be able to reproduce fast and clean the results that the Research team has already verified. 

### Notes on v0.1.0

- Due to Rust's ownership model, the Market struct executes the post=hook logic after completing the market order.
Is this the same as MGV? -> Maxence

- check real values for gas costs:
    - offer_write_cost 
    - taker_cost (gasbase)
    = order delete cost (to simulate MM than be both off/on chain)
- Finish kandel test comparing with python
    - check initial price grid prices and qty
    - check order book at t=0
    - check order book after 100 days