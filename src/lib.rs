use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::ValidAccountId;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, BorshStorageKey};
use std::cmp;

setup_alloc!();

pub type BookId = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Status {
    List,
    Read,
    Finished,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Book {
    book_id: Option<BookId>,
    account_id: Option<AccountId>,
    title: String,
    description: String,
    status: Status,
    image: String,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    books_by_owner_id: UnorderedMap<AccountId, UnorderedSet<BookId>>,
    books: UnorderedMap<BookId, Book>,
    books_len: u64,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    BooksByOwner,
    Books,
    BooksPerOwner { account_hash: Vec<u8> },
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            books_by_owner_id: UnorderedMap::new(StorageKey::BooksByOwner),
            books: UnorderedMap::new(StorageKey::Books),
            books_len: 0,
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn add_book(&mut self, mut book: Book) -> BookId {
        let account_id = env::predecessor_account_id();

        let current_book_id = format!("{}", self.books_len + 1);

        book.book_id = Some(current_book_id.clone());
        book.account_id = Some(account_id.clone());
        self.books.insert(&current_book_id, &book);

        let books_by_owner = self.books_by_owner_id.get(&account_id);
        match books_by_owner {
            Some(mut book_ids) => {
                book_ids.insert(&current_book_id);
                self.books_by_owner_id.insert(&account_id, &book_ids);
            }
            None => {
                let mut book_ids: UnorderedSet<BookId> =
                    UnorderedSet::new(StorageKey::BooksPerOwner {
                        account_hash: env::sha256(&account_id.as_bytes()),
                    });
                book_ids.insert(&current_book_id);
                self.books_by_owner_id.insert(&account_id, &book_ids);
            }
        }

        self.books_len += 1;

        return current_book_id;
    }

    pub fn update_book(&mut self, book_id: BookId, status: Status) -> Option<Book> {
        let account_id = env::predecessor_account_id();

        let book_ids = self.books_by_owner_id.get(&account_id).unwrap();
        if book_ids.contains(&book_id) {
            let mut book = self.books.get(&book_id).unwrap();
            book.status = status;
            self.books.insert(&book_id, &book);
            return Some(book);
        } else {
            panic!("Book does not exist");
        }
    }

    pub fn delete_book(&mut self, book_id: BookId) -> Option<Book> {
        let account_id = env::predecessor_account_id();

        let book_ids = self.books_by_owner_id.get(&account_id).unwrap();
        if book_ids.contains(&book_id) {
            let book = self.books.remove(&book_id);

            let mut book_ids = self.books_by_owner_id.get(&account_id).unwrap();
            book_ids.remove(&book_id);
            self.books_by_owner_id.insert(&account_id, &book_ids);

            return book;
        } else {
            panic!("Book does not exist");
        }
    }

    pub fn get_books(
        self,
        account_id: Option<ValidAccountId>,
        skip: u64,
        limit: Option<u64>,
    ) -> Option<Vec<Book>> {
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");


        if account_id.is_none() {
            let skip = cmp::min(self.books.len(), skip);

            return Some(
                self.books
                    .iter()
                    .skip(skip as usize)
                    .take(limit as usize)
                    .map(|(_, book)| book)
                    .collect(),
            );
        }

        let book_ids: UnorderedSet<BookId> = self
            .books_by_owner_id
            .get(&account_id.unwrap().to_string())
            .unwrap_or(UnorderedSet::new("".as_bytes()));

        let skip = cmp::min(self.books.len(), skip);

        return book_ids
            .iter()
            .skip(skip as usize)
            .take(limit)
            .map(|book_id| self.books.get(&book_id))
            .collect();
    }

    pub fn get_book(self, book_id: BookId) -> Book {
        self.books.get(&book_id).expect("Book does not exist")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn test_add_book() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Contract::default();
        let book_id = contract.add_book(Book {
            book_id: None,
            account_id: None,
            description: "Tutorial for mechanics".to_string(),
            image: "https://example.com".to_string(),
            status: Status::List,
            title: "Motorcycle Mechanics 101".to_string(),
        });

        let book = contract.get_book(book_id.clone());
        assert_eq!(book.book_id.unwrap(), book_id);
        assert_eq!(book.description, "Tutorial for mechanics".to_string());
        assert_eq!(book.image, "https://example.com".to_string());
        assert_eq!(book.title, "Motorcycle Mechanics 101".to_string());
    }

    #[test]
    fn test_update_book() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Contract::default();
        let book_id = contract.add_book(Book {
            book_id: None,
            account_id: None,
            description: "Tutorial for mechanics".to_string(),
            image: "https://example.com".to_string(),
            status: Status::List,
            title: "Motorcycle Mechanics 101".to_string(),
        });

        contract.update_book(book_id, Status::Read);
    }

    #[test]
    #[should_panic( expected = "Book does not exist" )]
    fn test_delete_book() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Contract::default();
        let book_id = contract.add_book(Book {
            book_id: None,
            account_id: None,
            description: "Tutorial for mechanics".to_string(),
            image: "https://example.com".to_string(),
            status: Status::List,
            title: "Motorcycle Mechanics 101".to_string(),
        });

        contract.delete_book(book_id.clone());

        let book = contract.get_book(book_id.clone());
    }
}
