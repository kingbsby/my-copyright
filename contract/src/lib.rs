use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{Vector, UnorderedSet, LookupMap, UnorderedMap};
use near_sdk::{json_types::U128, env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise, PromiseResult};
use near_sdk::serde::Serialize;

// use rand::{Rng, SeedableRng};

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub struct Picture {
    pub owner: AccountId,
    pub opus_name: String,
    pub opus_url: String,
    pub opus_hash: String,
    pub price: u128,
}

impl Picture {
    pub fn new(owner: AccountId, opus_name: String, opus_url: String, opus_hash: String, price: u128) -> Self {
        Self {
            owner: owner,
            opus_name: opus_name,
            opus_url: opus_url,
            opus_hash: opus_hash,
            price: price,
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct PictureStore {
    pub unit:u128,
    pub picture_list: UnorderedMap<String, Picture>
}

#[near_bindgen]
impl PictureStore {

    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            unit: 1_000_000_000_000_000_000_000,   // 0.001near
            picture_list: UnorderedMap::new(b"a".to_vec()),
        }
    }

    pub fn get_list(&self) -> Vec<Picture>{
        let mut result = vec![];
        for pic in self.picture_list.values() {
            result.push(pic);
        };
        result
    }

    pub fn add_picture(&mut self, opus_name: String, opus_url: String, opus_hash: String, price: u128) -> String{
        let account_id= env::signer_account_id();

        let pic_curr = self.picture_list.get(&opus_hash);
        let updated_pic = match pic_curr {
            Some(r) => {
                if account_id != r.owner && opus_hash == r.opus_hash {
                    None
                }else{
                    Some(Picture::new(account_id, opus_name, opus_url, opus_hash, price))
                }
            },
            None =>  Some(Picture::new(account_id, opus_name, opus_url, opus_hash, price))
        };

        match updated_pic {
            Some(r) => {self.picture_list.insert(&r.opus_hash, &r); "success".to_string()}
            None => "The Picture already exists".to_string()
        }
    }

    fn pay_seller(&mut self, account_id : AccountId, amount : u128) -> Promise {
        Promise::new(account_id.parse().unwrap())
            .transfer(amount * self.unit)
    }



    #[payable]
    pub fn buy(&mut self, pic_hash: String) -> String {
        //get buyer and seller account_id
        let buy_account_id= env::signer_account_id();
        let mut pic = self.picture_list.get(&pic_hash).expect("The Picture not exists");
        let sell_account_id = pic.owner;
        if sell_account_id == buy_account_id {
            panic! ("You cannot buy your own paintings");
        }

        // get price and check deposit
        let price = pic.price;
        println!("pic price {}, buy price {}", price, env::attached_deposit());
        // assert!(env::attached_deposit() == price, "Incorrect price : accountId {}, pic price {}, buy price {}",buy_account_id, price, env::attached_deposit());
        assert!(env::attached_deposit() == price * self.unit, "Incorrect price");

        // transfer near to seller
        // Promise::new(sell_account_id).transfer(price);
        println!("curr account {}, to account {}, price {}", &buy_account_id, &sell_account_id, &price);
        self.pay_seller(sell_account_id, price);

        // change the owner of picture to buyer
        pic.owner = buy_account_id.clone();
        self.picture_list.insert(&pic_hash, &pic);
        "Successful purchase".to_string()
    }
}


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::{testing_env, MockedBlockchain, VMContext};

    pub fn get_context(account_id: AccountId, block_timestamp: u64) -> VMContext {
        VMContext {
            current_account_id: account_id.clone(),
            signer_account_id: account_id.clone(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: account_id,
            input: vec![],
            block_index: 1,
            block_timestamp,
            epoch_height: 1,
            account_balance: 10u128.pow(26),
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit: 0,
            prepaid_gas: 300 * 10u64.pow(12),
            random_seed: vec![0, 2, 1, 3, 4, 5, 6, 7],
            is_view: false,
            output_data_receivers: vec![],
        }
    }

    #[test]
    fn test_first_add_picture() {
        let context = get_context("pic.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut picture_store = PictureStore::new();

        assert_eq!(picture_store.add_picture("werwer".to_string(), "asds".to_string(), "213555".to_string(), 100)
        , "success".to_string());
    }

    #[test]
    fn test_second_add_picture() {
        let context = get_context("pic.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut picture_store = PictureStore::new();
        picture_store.add_picture("pic1".to_string(), "pic1_url".to_string(), "123456".to_string(), 100);

        assert_eq!(picture_store.add_picture("pic2".to_string(), "pic2_url".to_string(), "123456".to_string(), 200)
        , "success".to_string());
    }

    #[test]
    fn test_add_picture_existed() {
        let mut context = get_context("user1.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut picture_store = PictureStore::new();
        picture_store.add_picture("pic1".to_string(), "pic1_url".to_string(), "123456".to_string(), 100);
        context.signer_account_id = "user2.test".to_string();
        testing_env!(context.clone());
        assert_eq!(picture_store.add_picture("pic2".to_string(), "pic2_url".to_string(), "123456".to_string(), 200)
        , "The Picture already exists".to_string());
    }

    #[test]
    #[should_panic(expected = r#"Incorrect price"#)]
    fn test_repeat_pay_failed() {
        let mut context = get_context("pic1.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut picture_store = PictureStore::new();
        let a : String = picture_store.add_picture("asd".to_string(), "asds".to_string(), "123456".to_string(), 100);
        println!("add asd:{}", a);
        context.signer_account_id = "pic2.text".to_string();
        context.attached_deposit = 50;
        testing_env!(context.clone());
        picture_store.buy("123456".to_string());
        // println!("buy result:{}", re);
        // assert_eq!(re, "Incorrect price");
    }

    #[test]
    fn test_repeat_pay_success() {
        let mut context = get_context("pic1.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut picture_store = PictureStore::new();
        let a : String = picture_store.add_picture("asd".to_string(), "asds".to_string(), "123456".to_string(), 1000);
        println!("add asd:{}", a);
        println!("seller before : {}", env::account_balance());

        context.signer_account_id = "pic2.text".to_string();
        context.account_balance = 9_000_000_000_000_000_000_000_000u128;
        context.attached_deposit = 1000 * 1_000_000_000_000_000_000_000;
        testing_env!(context.clone());
        picture_store.buy("123456".to_string());
        println!("buyer after : {}", env::account_balance());
        // println!("buy result:{}", re);
        // assert_eq!(re, "Successful purchase");
    }
}