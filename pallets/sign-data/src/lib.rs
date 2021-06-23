#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_health_ai::Config {
        /// 设备类型 1:手环 2:床垫 3:血糖仪 4:血压计 5:体温计 6:跌倒报警 7:电子围栏 8:其他
        type DeviceType: Parameter + Member + Default + Copy;
        /// 关系 1:本人 2:父亲 3:母亲 3:岳父 4:岳母 9:其他 因为亲属关系太多，就不用枚举了，前后端约定即可
        // type RelationType: Parameter + Member + Default + Copy;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);


    #[pallet::storage]
    #[pallet::getter(fn owned_devices)]
    /// 账户和设备关联关系 DeviceType u8
    pub type OwnedDevices<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, (T::RelationType, T::DeviceType), Vec<u8>>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 帐号绑定亲属设备成功. [who, relation,sn]
        RelationDeviceStored(T::AccountId, (T::RelationType, T::DeviceType), Vec<u8>),
        /// 帐号解绑亲属信息成功. [who, relation]
        RelationDeviceUnbind(T::AccountId, (T::RelationType, T::DeviceType)),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        /// 帐号没有绑定亲属关系
        RelationIsNotStored,
        /// 帐号没有绑定亲属设备
        RelationDeviceIsNotStored,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}


    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 绑定亲属设备
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn bind(origin: OriginFor<T>, relation_type: T::RelationType, device_type: T::DeviceType, sn: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let is_stored = pallet_health_ai::Module::<T>::relation_stored(&sender, &relation_type);
            //  检查是否已经绑定过亲属
            ensure!(is_stored, Error::<T>::RelationIsNotStored);
            OwnedDevices::<T>::insert(&sender, (relation_type, device_type), &sn);
            Self::deposit_event(Event::RelationDeviceStored(sender, (relation_type, device_type), sn));
            Ok(().into())
        }

        /// 解除绑定亲属设备
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn unbind(origin: OriginFor<T>, relation_type: T::RelationType, device_type: T::DeviceType) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(OwnedDevices::<T>::contains_key(&sender,(&relation_type, &device_type)), Error::<T>::RelationDeviceIsNotStored);
            OwnedDevices::<T>::remove(&sender, (&relation_type, &device_type));
            // 发布解除绑定事件
            Self::deposit_event(Event::RelationDeviceUnbind(sender, (relation_type, device_type)));
            Ok(().into())
        }
    }
}

impl<T: Config + pallet_health_ai::Config> Pallet<T> {}
