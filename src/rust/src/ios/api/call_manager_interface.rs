//
// Copyright (C) 2019, 2020 Signal Messenger, LLC.
// All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
//

//! iOS Call Manager Interface

use std::ffi::c_void;
use std::{fmt, ptr, slice, str};

use libc::size_t;

use crate::ios::call_manager;
use crate::ios::call_manager::IOSCallManager;
use crate::ios::ios_util::*;
use crate::ios::logging::IOSLogger;

use crate::common::DeviceId;

use crate::webrtc::ice_candidate::IceCandidate;

///
#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[allow(non_snake_case)]
pub struct AppObject {
    pub ptr: *mut c_void,
}

// Add an empty Send trait to allow transfer of ownership between threads.
unsafe impl Send for AppObject {}

// Add an empty Sync trait to allow access from multiple threads.
unsafe impl Sync for AppObject {}

impl From<AppObject> for *mut c_void {
    fn from(item: AppObject) -> Self {
        item.ptr
    }
}

impl From<AppObject> for *const c_void {
    fn from(item: AppObject) -> Self {
        item.ptr
    }
}

impl From<&AppObject> for *mut c_void {
    fn from(item: &AppObject) -> Self {
        item.ptr
    }
}

impl AppObject {
    pub fn new(ptr: *mut c_void) -> Self {
        Self { ptr }
    }
}

impl From<*mut c_void> for AppObject {
    fn from(item: *mut c_void) -> Self {
        AppObject::new(item)
    }
}

impl From<*const c_void> for AppObject {
    fn from(item: *const c_void) -> Self {
        AppObject::new(item as *mut c_void)
    }
}

/// Structure for passing Ice Candidates to/from Swift.
#[repr(C)]
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct AppIceCandidate {
    pub sdpMid:        AppByteSlice,
    pub sdpMLineIndex: i32,
    pub sdp:           AppByteSlice,
}

/// Structure for passing multiple Ice Candidates to Swift.
#[repr(C)]
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct AppIceCandidateArray {
    pub candidates: *const AppIceCandidate,
    pub count:      size_t,
}

/// Structure for passing connection details from the application.
#[repr(C)]
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct AppConnectionInterface {
    pub object:  *mut c_void,
    pub pc:      *mut c_void,
    /// Swift object clean up method.
    pub destroy: extern "C" fn(object: *mut c_void),
}

// Add an empty Send trait to allow transfer of ownership between threads.
unsafe impl Send for AppConnectionInterface {}

// Add an empty Sync trait to allow access from multiple threads.
unsafe impl Sync for AppConnectionInterface {}

impl fmt::Display for AppConnectionInterface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Rust owns the connection details object from Swift. Drop it when it
// goes out of scope.
impl Drop for AppConnectionInterface {
    fn drop(&mut self) {
        (self.destroy)(self.object);
    }
}

/// Structure for holding call context details on behalf of the application.
#[repr(C)]
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct AppCallContext {
    pub object:  *mut c_void,
    /// Swift object clean up method.
    pub destroy: extern "C" fn(object: *mut c_void),
}

// Add an empty Send trait to allow transfer of ownership between threads.
unsafe impl Send for AppCallContext {}

// Add an empty Sync trait to allow access from multiple threads.
unsafe impl Sync for AppCallContext {}

impl fmt::Display for AppCallContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Rust owns the connection details object from Swift. Drop it when it
// goes out of scope.
impl Drop for AppCallContext {
    fn drop(&mut self) {
        (self.destroy)(self.object);
    }
}

/// Structure for passing media stream instances from the application.
#[repr(C)]
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct AppMediaStreamInterface {
    pub object:            *mut c_void,
    /// Swift object clean up method.
    pub destroy:           extern "C" fn(object: *mut c_void),
    /// Returns a pointer to a RTCMediaStream object.
    pub createMediaStream:
        extern "C" fn(object: *mut c_void, nativeStream: *mut c_void) -> *mut c_void,
}

// Add an empty Send trait to allow transfer of ownership between threads.
unsafe impl Send for AppMediaStreamInterface {}

// Add an empty Sync trait to allow access from multiple threads.
unsafe impl Sync for AppMediaStreamInterface {}

impl fmt::Display for AppMediaStreamInterface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Rust owns the connection details object from Swift. Drop it when it
// goes out of scope.
impl Drop for AppMediaStreamInterface {
    fn drop(&mut self) {
        (self.destroy)(self.object);
    }
}

#[repr(C)]
#[derive(Debug)]
#[allow(non_snake_case)]
/// iOS Interface for communicating with the Swift application.
pub struct AppInterface {
    /// Raw Swift object pointer.
    pub object:                       *mut c_void,
    /// Swift object clean up method.
    pub destroy:                      extern "C" fn(object: *mut c_void),
    ///
    pub onStartCall:
        extern "C" fn(object: *mut c_void, remote: *const c_void, callId: u64, isOutgoing: bool),
    /// Swift event callback method.
    pub onEvent: extern "C" fn(object: *mut c_void, remote: *const c_void, event: i32),
    ///
    pub onSendOffer: extern "C" fn(
        object: *mut c_void,
        callId: u64,
        remote: *const c_void,
        deviceId: u32,
        broadcast: bool,
        offer: AppByteSlice,
    ),
    ///
    pub onSendAnswer: extern "C" fn(
        object: *mut c_void,
        callId: u64,
        remote: *const c_void,
        deviceId: u32,
        broadcast: bool,
        answer: AppByteSlice,
    ),
    ///
    pub onSendIceCandidates: extern "C" fn(
        object: *mut c_void,
        callId: u64,
        remote: *const c_void,
        deviceId: u32,
        broadcast: bool,
        candidates: *const AppIceCandidateArray,
    ),
    ///
    pub onSendHangup: extern "C" fn(
        object: *mut c_void,
        callId: u64,
        remote: *const c_void,
        deviceId: u32,
        broadcast: bool,
    ),
    ///
    pub onSendBusy: extern "C" fn(
        object: *mut c_void,
        callId: u64,
        remote: *const c_void,
        deviceId: u32,
        broadcast: bool,
    ),
    ///
    pub onCreateConnectionInterface: extern "C" fn(
        object: *mut c_void,
        observer: *mut c_void,
        deviceId: u32,
        context: *mut c_void,
    ) -> AppConnectionInterface,
    /// Request that the application create an application Media Stream object
    /// associated with the given application Connection object.
    pub onCreateMediaStreamInterface:
        extern "C" fn(object: *mut c_void, connection: *mut c_void) -> AppMediaStreamInterface,
    ///
    pub onConnectMedia: extern "C" fn(
        object: *mut c_void,
        remote: *const c_void,
        context: *mut c_void,
        stream: *const c_void,
    ),
    ///
    pub onCompareRemotes:
        extern "C" fn(object: *mut c_void, remote1: *const c_void, remote2: *const c_void) -> bool,
    ///
    pub onCallConcluded:              extern "C" fn(object: *mut c_void, remote: *const c_void),
}

// Add an empty Send trait to allow transfer of ownership between threads.
unsafe impl Send for AppInterface {}

// Add an empty Sync trait to allow access from multiple threads.
unsafe impl Sync for AppInterface {}

// Rust owns the interface object from Swift. Drop it when it goes out
// of scope.
impl Drop for AppInterface {
    fn drop(&mut self) {
        (self.destroy)(self.object);
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcInitialize(logObject: IOSLogger) -> *mut c_void {
    match call_manager::initialize(logObject) {
        Ok(_v) => {
            // Return non-null pointer to indicate success.
            1 as *mut c_void
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcCreate(
    appCallManager: *mut c_void,
    appInterface: AppInterface,
) -> *mut c_void {
    match call_manager::create(appCallManager, appInterface) {
        Ok(v) => v,
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcCall(callManager: *mut c_void, appRemote: *const c_void) -> *mut c_void {
    match call_manager::call(callManager as *mut IOSCallManager, appRemote) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcProceed(
    callManager: *mut c_void,
    callId: u64,
    appCallContext: AppCallContext,
    appRemoteDevices: *const u32,
    appRemoteDevicesLen: size_t,
) -> *mut c_void {
    // Convert the remoteDevices list from a u32 array to a vector.
    let device_slice =
        unsafe { slice::from_raw_parts(appRemoteDevices, appRemoteDevicesLen as usize) };

    match call_manager::proceed(
        callManager as *mut IOSCallManager,
        callId,
        appCallContext,
        device_slice.to_vec(),
    ) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcMessageSent(callManager: *mut c_void, callId: u64) -> *mut c_void {
    match call_manager::message_sent(callManager as *mut IOSCallManager, callId) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcMessageSendFailure(callManager: *mut c_void, callId: u64) -> *mut c_void {
    match call_manager::message_send_failure(callManager as *mut IOSCallManager, callId) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcHangup(callManager: *mut c_void) -> *mut c_void {
    match call_manager::hangup(callManager as *mut IOSCallManager) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcReceivedAnswer(
    callManager: *mut c_void,
    callId: u64,
    remoteDevice: u32,
    answer: AppByteSlice,
) -> *mut c_void {
    // Build the Rust string.
    let answer_bytes = unsafe { slice::from_raw_parts(answer.bytes, answer.len as usize) };

    match str::from_utf8(answer_bytes) {
        Ok(session_desc) => {
            match call_manager::received_answer(
                callManager as *mut IOSCallManager,
                callId,
                remoteDevice as DeviceId,
                session_desc,
            ) {
                Ok(_v) => {
                    // Return the object reference back as indication of success.
                    callManager
                }
                Err(_e) => ptr::null_mut(),
            }
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcReceivedOffer(
    callManager: *mut c_void,
    callId: u64,
    appRemote: *const c_void,
    remoteDevice: u32,
    offer: AppByteSlice,
    timestamp: u64,
) -> *mut c_void {
    // Build the Rust string.
    let offer_bytes = unsafe { slice::from_raw_parts(offer.bytes, offer.len as usize) };

    match str::from_utf8(offer_bytes) {
        Ok(session_desc) => {
            match call_manager::received_offer(
                callManager as *mut IOSCallManager,
                callId,
                appRemote,
                remoteDevice as DeviceId,
                session_desc,
                timestamp,
            ) {
                Ok(_v) => {
                    // Return the object reference back as indication of success.
                    callManager
                }
                Err(_e) => ptr::null_mut(),
            }
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcReceivedIceCandidate(
    callManager: *mut c_void,
    callId: u64,
    remoteDevice: u32,
    app_candidate: AppIceCandidate,
) -> *mut c_void {
    let mut ice_candidates = Vec::new();

    // Build the Rust strings.
    let sdp_bytes =
        unsafe { slice::from_raw_parts(app_candidate.sdp.bytes, app_candidate.sdp.len as usize) };

    let sdp_string = match str::from_utf8(sdp_bytes) {
        Ok(sdp_desc) => sdp_desc,
        Err(_e) => "",
    };

    let sdp_mid_bytes = unsafe {
        slice::from_raw_parts(
            app_candidate.sdpMid.bytes,
            app_candidate.sdpMid.len as usize,
        )
    };

    let sdp_mid_string = match str::from_utf8(sdp_mid_bytes) {
        Ok(sdp_mid_desc) => sdp_mid_desc,
        Err(_e) => "",
    };

    if sdp_string.is_empty() || sdp_mid_string.is_empty() {
        warn!("ringrtcReceivedIceCandidates: No valid candidates!");
        return ptr::null_mut();
    }

    let ice_candidate = IceCandidate::new(
        sdp_mid_string.to_string(),
        app_candidate.sdpMLineIndex,
        sdp_string.to_string(),
    );
    ice_candidates.push(ice_candidate);

    match call_manager::received_ice_candidates(
        callManager as *mut IOSCallManager,
        callId,
        remoteDevice as DeviceId,
        ice_candidates.to_vec(),
    ) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcReceivedHangup(
    callManager: *mut c_void,
    callId: u64,
    remoteDevice: u32,
) -> *mut c_void {
    match call_manager::received_hangup(
        callManager as *mut IOSCallManager,
        callId,
        remoteDevice as DeviceId,
    ) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcReceivedBusy(
    callManager: *mut c_void,
    callId: u64,
    remoteDevice: u32,
) -> *mut c_void {
    match call_manager::received_busy(
        callManager as *mut IOSCallManager,
        callId,
        remoteDevice as DeviceId,
    ) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcAccept(callManager: *mut c_void, callId: u64) -> *mut c_void {
    match call_manager::accept_call(callManager as *mut IOSCallManager, callId) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcGetActiveConnection(callManager: *mut c_void) -> *mut c_void {
    match call_manager::get_active_connection(callManager as *mut IOSCallManager) {
        Ok(v) => v,
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcGetActiveCallContext(callManager: *mut c_void) -> *mut c_void {
    match call_manager::get_active_call_context(callManager as *mut IOSCallManager) {
        Ok(v) => v,
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcSetVideoEnable(callManager: *mut c_void, enable: bool) -> *mut c_void {
    match call_manager::set_video_enable(callManager as *mut IOSCallManager, enable) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcDrop(callManager: *mut c_void, callId: u64) -> *mut c_void {
    match call_manager::drop_call(callManager as *mut IOSCallManager, callId) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcReset(callManager: *mut c_void) -> *mut c_void {
    match call_manager::reset(callManager as *mut IOSCallManager) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn ringrtcClose(callManager: *mut c_void) -> *mut c_void {
    match call_manager::close(callManager as *mut IOSCallManager) {
        Ok(_v) => {
            // Return the object reference back as indication of success.
            callManager
        }
        Err(_e) => ptr::null_mut(),
    }
}
