#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use serde::{Deserialize, Deserializer, Serialize};
use sp_std::vec::Vec;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Default, Clone, PartialEq)]
///绑定对象信息
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
        /// 关系 1:本人 2:父亲 3:母亲 3:岳父 4:岳母 9:其他 因为亲属关系太多，就不用枚举了，前后端约定即可
        type RelationType: Parameter + Member + Default + Copy;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);


    #[pallet::storage]
    #[pallet::getter(fn relation_id_cards)]
    /// 账户和亲属关联关系
    pub type Relations<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::RelationType, PersonInfo>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 帐号绑定亲属信息成功. [who, PersonInfo]
        RelationStored(T::AccountId, T::RelationType, PersonInfo),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// 必填字段不能为空
        NoneValue,
        /// 存储越界
        StorageOverflow,
        /// json格式数据转换异常
        JsonParamError,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 绑定亲属信息 json方式
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn bind(origin: OriginFor<T>, relation_type: T::RelationType, json: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            /*
            let data = r#"
            {
                "name": "luffy",
                "id_card": "43",
                "height": 43,
                "weight": 43,
                "chronic": "123456"
            }"#;
            let ps_info: PersonInfo = serde_json::from_str(&data).map_err(|_| <Error<T>>::JsonParamError)?;
            */
            // let ps_info: PersonInfo = serde_json::from_slice(&json).unwrap();
            // 检查json格式是否合法，不合法抛出异常
            let ps_info: PersonInfo = serde_json::from_slice(&json).map_err(|_| <Error<T>>::JsonParamError)?;
            Relations::<T>::insert(&sender, relation_type, &ps_info);
            // 发布绑定成功事件
            Self::deposit_event(Event::RelationStored(sender, relation_type, ps_info));
            Ok(().into())
        }

        /// 绑定亲属信息 struct方式
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn bind_info(origin: OriginFor<T>, relation_type: T::RelationType, ps_info: PersonInfo) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            Relations::<T>::insert(&sender, relation_type, &ps_info);
            // 发布绑定成功事件
            Self::deposit_event(Event::RelationStored(sender, relation_type, ps_info));
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// 账户是否绑定种亲属关系
    pub fn relation_stored(owner: &T::AccountId, relation_type: &T::RelationType) -> bool {
        return Relations::<T>::contains_key(owner, relation_type);
    }
}

