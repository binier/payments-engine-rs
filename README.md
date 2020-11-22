## Calculate accounts' balances based on list of transactions

Currently only works with **csv** files.

To run dev version simply use: `cargo run my-input.csv`

#### Concurrent mode

Supports concurrent mode, which distributes clients across different
threads to go though transactions faster.

In order to run in concurrent mode, pass in the option to the cli:
```bash
cargo run -- --concurrent my-input.csv
```

It uses client's id to decide to which thread to send it to. Simply
applies **modulo(%)** operator to the `client_id` and based on result sends
to corresponding thread.

So thread to which client goes to **=** `client_id % thread_count`. `thread_count`
depends on **cpu core count**, can be customized from code though.

This means that in very rare and **worst cast scenario**, all clients will
go to the same thread. In such case it will be a little bit slower than
single threaded version, since we have to pay cost for sending transactions
through the channel.

For example if we have **8** cores, hence 8 threads and we receive 
transactions for clients: `1, 9, 17, 25, 33, ...` will all run on the same
thread.
