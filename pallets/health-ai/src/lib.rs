#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, ensure, RuntimeDebug};
use serde::{Deserialize, Deserializer, Serialize};
use sp_std::vec::Vec;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[derive(Debug, Serialize, Deserialize, Encode, Decode, Default)]
pub struct PersonInfo {
    #[serde(deserialize_with = "de_string_to_bytes")]
    name: Vec<u8>,
    //姓名
    #[serde(deserialize_with = "de_string_to_bytes")]
    id_card: Vec<u8>,
    //身份证号码
    height: u16,
    //身高 mm
    weight: u16,
    //体重 g
    #[serde(deserialize_with = "de_string_to_bytes")]
    chronic: Vec<u8>, //慢性病
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de> {
    let s: &str = Deserialize::deserialize(de)?;
    Ok(s.as_bytes().to_vec())
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 关系 1:本人 2:父亲 3:母亲 3:岳父 4:岳母 9:其他
        type RelationType: Parameter + Member + Default + Copy;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);


    #[pallet::storage]
    #[pallet::getter(fn relation_id_cards)]
    /// 账户和身份证关联关系 RelationType u8
    pub type RelationIdCards<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::RelationType, PersonInfo>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored(u32, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        JsonParamError,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 绑定亲属信息
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn bind(origin: OriginFor<T>, relation_type: T::RelationType, json: Vec<u8>) -> DispatchResultWithPostInfo {
            let _sender = ensure_signed(origin)?;
            // let data1 = str::from_utf8(&json).map_err(|_| <Error<T>>::JsonParamError)?;
            // let data1 = String::from_utf8(json).map_err(|_| <Error<T>>::JsonParamError)?;
            let data = r#"
            {
                "name": "luffy",
                "id_card": "43",
                "height": 43,
                "weight": 43,
                "chronic": "123456"
            }"#;
            let ps_info: PersonInfo = serde_json::from_str(&data).map_err(|_| <Error<T>>::JsonParamError)?;
            RelationIdCards::<T>::insert(&_sender, relation_type, ps_info);
            Ok(().into())
        }
    }
}
