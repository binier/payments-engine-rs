use std::thread;
use std::sync::mpsc;

use num_cpus;

use crate::types::ClientID;
use crate::transaction::Transaction;
use crate::account::Account;
use crate::bank::Bank;
use crate::basic_bank::BasicBank;

struct BankThread {
    pub thread: thread::JoinHandle<BasicBank>,
    pub tx: mpsc::Sender<Transaction>,
}


/// Stores and manages accounts in the bank **Concurrently**.
///
/// It simply manages multiple subbanks each in it's own
/// thread. Then based on hash of the `client_id`, `ConcurrentBank`
/// decides to which subbank transaction should go to. This
/// way each subbank has a **dedicated only to it** set of clients.
pub struct ConcurrentBank {
    threads: Vec<BankThread>,
    count: usize,
}

impl Default for ConcurrentBank {
    fn default() -> Self {
        Self::new()
    }
}

impl Bank for ConcurrentBank {
    type AccountsIter = Box<dyn Iterator<Item = Account>>;
    // TODO: propagate error from apply_tx.
    /// Apply `Transaction` to the `Account` in `Bank`.
    fn apply_tx<T: Into<Transaction>>(&mut self, tx: T) -> Result<(), ()> {
        let tx: Transaction = tx.into();
        let bank_thread = self.get_thread_for_client_mut(tx.get_client_id());

        let _ = bank_thread.tx.send(tx);
        Ok(())
    }

    /// Consumes `ConcurrentBank` and **Blocks** untill all threads finish.
    /// Outputs `Account` iterator.
    fn into_accounts_iter(self) -> Self::AccountsIter {
        let iter = self.threads.into_iter()
            .map(|bank_thread| {
                let BankThread { thread, tx } = bank_thread;
                // drop `Sender` to let thread no that it's
                // work is finished and it can return.
                drop(tx);
                thread.join().unwrap()
            })
            .map(|bank| bank.into_accounts_iter())
            .flatten();
        Box::new(iter)
    }
}

impl ConcurrentBank {
    /// Create new empty bank
    pub fn new() -> Self {
        Self::new_with_thread_count(num_cpus::get())
    }

    /// Bank with custom thread count. `Default` is
    /// [number of cpu cores](std::env::concurrency_hint)
    pub fn new_with_thread_count(count: usize) -> Self {
        Self {
            count,
            threads: (0..count).map(|_| {
                let (tx, rx) = mpsc::channel();
                let thread = thread::spawn(move || {
                    let mut bank = BasicBank::new();
                    loop {
                        match rx.recv() {
                            Ok(transaction) => {
                                // ignore result
                                let _ = bank.apply_tx(transaction);
                            },
                            Err(mpsc::RecvError) => break,
                        }
                    }
                    bank
                });

                BankThread { tx, thread }
            }).collect(),
        }
    }

    /// Get's a thread that stores account for the following client.
    /// **Will** always return same value so only one thread/bank
    /// manages same client.
    fn get_thread_for_client_mut(
        &mut self,
        client_id: ClientID
    ) -> &mut BankThread {
        &mut self.threads[(client_id as usize) % self.count]
    }
}
