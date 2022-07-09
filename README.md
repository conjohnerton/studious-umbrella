# What assumptions did I make?
    - If an account was locked, all transactions to that account should be rejected.
    - If a dispute was made to a withdrawal tx, the transaction would be rejected. Not sure how else we could handle that with the current problem definition.


# Error Handling:
    - I used anyhow, as it's what I've used before to make errors easier to handle. It wasn't REALLY useful in this case, but if we wanted a new error to bubble up from some deep function call, it would be trivial with anyhow.

# Correctness:
    - I never assumed that a value would be present when retrieving data from the hashmaps, I just didn't do anything in those cases.
        - If expanding this, would probably throw errors in that case, but didn't want to muddy the output.
    - When inserting any data into a map, I did not re-use references. It would have avoided allocations yeah, but in this case it probably wouldn't be a big deal, since Accounts are small at the moment.

# What could be done better?
    - We could use a db for the client data and tx list, instead of an in-memory hashtable.
    - Could asyncronously stream lines of a file and chain it to a csv stream. At first look, might be able to use libs like tokio, tokio_file, async-stream, and csv_stream to do something like this.
    - If we wanted to make this an actual group of services, we could do the following to scale up a bit.
        - CSV Saver Service(s)
            - Receives CSV's from multiple sources, splits the csv's into transactions, and saves transactions into a tx queue.
            - Lives behind a load balancer that points to one or multiple CSV Savers.
        - Transaction Doer Service(s)
            - Polls the db for new tx's and does the tx.
            - If a tx succeeds
                - atomic (update the client_account and tx_info db with the details and mark the tx as done).
            - If a tx fails due to an account being locked
                - the tx is marked as done in the db.
            - If a tx fails due to a withdrawal not having enough available funds to complete
                - send it back to the tx queue
            - If a tx fails due to a there not being an active dispute on the tx being resolved or chargeback'ed
                - The tx is sent back to the tx queue
        - This makes some assumptions about how the system might work.
        - Skips over making sure that each Transaction Doer either completes a tx or marks it completed.
        - Skips over making sure that each CSV Saver adds all a CSV's transactions to the tx queue.

# To run the test cases:
```
for f in resources/*; do
  cargo run -- $f
done
```
Thanks!