// use crate::{
//     descriptor_table::{DescriptorTable, SharedDescriptorTable},
//     device::Device,
//     driver::Driver,
//     error::Result,
//     rings::{AvailableRing, SharedAvailableRing, SharedUsedRing, UsedRing},
// };
// use std::sync::{Arc, Condvar, Mutex};
//
// pub fn _run() -> Result<()> {
//     // define our structures
//     // the table is local state to the driver, while the rings are shared between the driver and the
//     // device. Therefore, we'll add standard synchronization on the rings for now.
//     // TODO: Need to ensure the order that we grab the locks in is the same if we ever require more
//     // than one lock to be held to avoid deadlock.
//     let table: SharedDescriptorTable = Arc::new(Mutex::new(DescriptorTable::default()));
//     let available_ring: SharedAvailableRing = Arc::new(Mutex::new(AvailableRing::default()));
//     let used_ring: SharedUsedRing = Arc::new(Mutex::new(UsedRing::default()));
//     let common_state: SharedCommonState = Arc::new(Mutex::new(CommonState::default()));
//     let driver_condvar = Arc::new(Condvar::new());
//
//     // now we can spawn our mock device that grabs from the available_ring
//     let handle = std::thread::spawn(move || {
//         let device = Device::new(
//             table.clone(),
//             available_ring.clone(),
//             used_ring.clone(),
//             0,
//             0,
//             driver_condvar.clone(),
//             common_state.clone(),
//         );
//         device.run()
//     });
//
//     // now we can pretend to be the driver
//     let driver = Driver::new(table, available_ring, used_ring, common_state);
//     driver.
//
//
// }
