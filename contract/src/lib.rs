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

pub trait Transac {
    fn pay(&mut self) -> String;
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

        let pic = Picture::new(account_id, opus_name, opus_url, opus_hash, price);

        match self.picture_list.get(&pic.opus_hash) {
            Some(_r) => return "The picture already exists".to_string(),
            None =>  "The picture not exists".to_string()
        };

        self.picture_list.insert(&pic.opus_hash, &pic);
        "success".to_string()
    }

    pub fn pay_seller(&mut self, account_id : AccountId, amount : u128) -> Promise {
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
    fn test_repeat_add_picture() {
        let context = get_context("pic.test".to_string(), 3_600_000_000_000);
        testing_env!(context.clone());
        let mut picture_store = PictureStore::new();
        picture_store.add_picture("asd".to_string(), "asds".to_string(), "123456".to_string(), 100);

        assert_eq!(picture_store.add_picture("asdsd".to_string(), "asds".to_string(), "123456".to_string(), 100)
        , "The picture already exists".to_string());
        assert_eq!(picture_store.add_picture("werwer".to_string(), "asds".to_string(), "213555".to_string(), 100)
        , "success".to_string());
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

    // #[test]
    // fn test_pk() {
    //     let mut context = get_context("digital1.test".to_string(), 3_600_000_000_002);
    //     testing_env!(context.clone());
    //     let mut contract = DigitalCenter::new();

    //     assert_eq!(contract.add_first(), "success");

    //     context.signer_account_id = "digital2.test".to_string();
    //     testing_env!(context.clone());
    //     assert_eq!(contract.add_first(), "success");
        
    //     context.signer_account_id = "digital1.test".to_string();
    //     testing_env!(context.clone());
    //     assert_eq!(contract.add_first(), "You already have more than one digital");

    //     assert_eq!(contract.pk(0, 1), "you are lose, target get your digital!");

    //     assert_eq!(contract.pk(0, 1), "you are not own_digital owner");

    //     assert_eq!(contract.add_first(), "success");

    //     context.random_seed = vec![0, 1, 2, 3, 4, 5, 6, 7];
    //     testing_env!(context.clone());

    //     assert_eq!(contract.pk(2, 1), "you are win, target digital level minus 1!");
    //     assert_eq!(contract.pk(2, 1), "you are win, get target digital!");
    //     context.signer_account_id = "digital2.test".to_string();
    //     testing_env!(context.clone());
    //     assert_eq!(contract.pk(1, 2), "you are not own_digital owner");
    // }

    // #[test]
    // fn test_pk_not_own() {
    //     let mut context = get_context("digital1.test".to_string(), 3_600_000_000_000);
    //     testing_env!(context.clone());
    //     let mut contract = DigitalCenter::new();

    //     assert_eq!(contract.add_first(), "success".to_string());

    //     context.signer_account_id = "digital2.test".to_string();
    //     testing_env!(context.clone());
    //     assert_eq!(contract.add_first(), "success".to_string());
        
    //     context.signer_account_id = "digital1.test".to_string();
    //     testing_env!(context.clone());
    //     assert_eq!(contract.pk(1, 0), "you are not own_digital owner");
    // }

    // #[test]
    // fn get_digitals_work(){
    //     let mut context = get_context("digital1.test".to_string(), 3_600_000_000_000);
    //     testing_env!(context.clone());
    //     let mut contract = DigitalCenter::new();

    //     let aa: Vec<u64> = Vec::new();
    //     assert_eq!(contract.get_digitals("digital1.test".to_string()), aa);
    //     assert_eq!(contract.add_first(), "success".to_string());
    //     assert_eq!(contract.get_digitals("digital1.test".to_string()), vec![0]);
    // }

    // #[test]
    // fn levelup(){
    //     let mut context = get_context("digital1.test".to_string(), 3_600_000_000_000);
    //     testing_env!(context.clone());
    //     let mut contract = DigitalCenter::new();
    //     assert_eq!(contract.add_first(), "success".to_string());
    //     context.attached_deposit = 50;
    //     testing_env!(context.clone());
    //     assert_eq!(contract.levelup(0), "levelup success");

        
    // }

    // #[test]
    // #[should_panic(expected = "Deposit is too low")]
    // fn levelup_panic(){
    //     let mut context = get_context("digital1.test".to_string(), 3_600_000_000_000);
    //     testing_env!(context.clone());
    //     let mut contract = DigitalCenter::new();
    //     context.attached_deposit = 5;
    //     testing_env!(context.clone());
    //     contract.levelup(0);
    // }
}