use std::thread;
use std::sync::mpsc;

use crate::types::ClientID;
use crate::transaction::Transaction;
use crate::account::Account;
use crate::bank::Bank;
use crate::basic_bank::BasicBank;

struct BankThread {
    thread: Option<thread::JoinHandle<BasicBank>>,
    sender: Option<crossbeam_channel::Sender<Transaction>>,
}

impl BankThread {
    pub fn new() -> Self {
        let (sender, rx) = crossbeam_channel::unbounded();
        let thread = thread::spawn(move || {
            let mut bank = BasicBank::new();
            while let Ok(tx) = rx.recv() {
                // ignore result
                let _ = bank.apply_tx(tx);
            }
            bank
        });

        BankThread {
            sender: Some(sender),
            thread: Some(thread),
        }
    }

    pub fn apply_tx(&mut self, tx: Transaction) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(tx);
        }
    }

    pub fn join(&mut self) -> Option<BasicBank> {
        match (self.thread.take(), self.sender.take()) {
            (Some(thread), Some(sender)) => {
                // drop `Sender` to let thread no that it's
                // work is finished and it can return.
                drop(sender);
                thread.join().ok()
            },
            _ => None,
        }
    }
}

impl Drop for BankThread {
    fn drop(&mut self) {
        let _ = self.join();
    }
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

        bank_thread.apply_tx(tx);
        Ok(())
    }

    /// Consumes `ConcurrentBank` and **Blocks** untill all threads finish.
    /// Outputs `Account` iterator.
    fn into_accounts_iter(self) -> Self::AccountsIter {
        let iter = self.into_inner_banks()
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
            threads: (0..count)
                .map(|_| BankThread::new()).collect(),
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

    fn into_inner_banks(self) -> impl Iterator<Item = BasicBank> {
        self.threads.into_iter()
            .map(|mut bank_thread| bank_thread.join().unwrap())
    }
}
