// Copyright © 2017 Mozilla Foundation
//
// This program is made available under an ISC-style license.  See the
// accompanying file LICENSE for details.

use ::*;
use ffi;
use std::ffi::CStr;
use std::os::raw::{c_int, c_void};
use std::ptr;
use util::UnwrapCStr;

#[repr(C)]
#[derive(Debug)]
pub struct Context(*mut ffi::pa_context);

impl Context {
    pub fn new<'a, OPT>(api: &MainloopApi, name: OPT) -> Self
        where OPT: Into<Option<&'a CStr>>
    {
        unsafe { Context(ffi::pa_context_new(api.raw_mut(), name.unwrap_cstr())) }
    }

    #[doc(hidden)]
    pub fn raw_mut(&self) -> &mut ffi::pa_context {
        unsafe { &mut *self.0 }
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn clear_state_callback(&self) {
        unsafe {
            ffi::pa_context_set_state_callback(self.raw_mut(), None, ptr::null_mut());
        }
    }

    pub fn set_state_callback<CB>(&self, _: CB, userdata: *mut c_void)
        where CB: Fn(&Context, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(c: *mut ffi::pa_context, userdata: *mut c_void)
            where F: Fn(&Context, *mut c_void)
        {
            let ctx = context::from_raw_ptr(c);
            let result = ::std::mem::transmute::<_, &F>(&())(&ctx, userdata);
            ::std::mem::forget(ctx);
            result
        }

        unsafe {
            ffi::pa_context_set_state_callback(self.raw_mut(), Some(wrapped::<CB>), userdata);
        }
    }

    pub fn errno(&self) -> ffi::pa_error_code_t {
        unsafe { ffi::pa_context_errno(self.raw_mut()) }
    }

    pub fn get_state(&self) -> ContextState {
        ContextState::try_from(unsafe {
            ffi::pa_context_get_state(self.raw_mut())
        }).expect("pa_context_get_state returned invalid ContextState")
    }

    pub fn connect<'a, OPT>(&self,
                            server: OPT,
                            flags: ContextFlags,
                            api: *const ffi::pa_spawn_api)
                            -> Result<()>
        where OPT: Into<Option<&'a CStr>>
    {
        let r = unsafe {
            ffi::pa_context_connect(self.raw_mut(),
                                    server.into().unwrap_cstr(),
                                    flags.into(),
                                    api)
        };
        error_result!((), r)
    }

    pub fn disconnect(&self) {
        unsafe {
            ffi::pa_context_disconnect(self.raw_mut());
        }
    }


    pub fn drain<CB>(&self, _: CB, userdata: *mut c_void) -> Result<Operation>
        where CB: Fn(&Context, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(c: *mut ffi::pa_context, userdata: *mut c_void)
            where F: Fn(&Context, *mut c_void)
        {
            let ctx = context::from_raw_ptr(c);
            let result = ::std::mem::transmute::<_, &F>(&())(&ctx, userdata);
            ::std::mem::forget(ctx);
            result
        }

        let o = unsafe { ffi::pa_context_drain(self.raw_mut(), Some(wrapped::<CB>), userdata) };
        if o.is_null() {
            return Err(ErrorCode::from_error_code(self.errno()));
        }
        Ok(unsafe { operation::from_raw_ptr(o) })
    }

    pub fn rttime_new<CB>(&self,
                          usec: USec,
                          _: CB,
                          userdata: *mut c_void)
                          -> *mut ffi::pa_time_event
        where CB: Fn(&MainloopApi, *mut ffi::pa_time_event, &TimeVal, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(a: *mut ffi::pa_mainloop_api,
                                        e: *mut ffi::pa_time_event,
                                        tv: *const TimeVal,
                                        userdata: *mut c_void)
            where F: Fn(&MainloopApi, *mut ffi::pa_time_event, &TimeVal, *mut c_void)
        {
            let api = mainloop_api::from_raw_ptr(a);
            let timeval = &*tv;
            let result = ::std::mem::transmute::<_, &F>(&())(&api, e, timeval, userdata);
            ::std::mem::forget(api);
            result
        }

        unsafe { ffi::pa_context_rttime_new(self.raw_mut(), usec, Some(wrapped::<CB>), userdata) }
    }

    pub fn get_server_info<CB>(&self, _: CB, userdata: *mut c_void) -> Result<Operation>
        where CB: Fn(&Context, &ServerInfo, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(c: *mut ffi::pa_context,
                                        i: *const ffi::pa_server_info,
                                        userdata: *mut c_void)
            where F: Fn(&Context, &ServerInfo, *mut c_void)
        {
            debug_assert_ne!(i, ptr::null_mut());
            let info = &*i;
            let ctx = context::from_raw_ptr(c);
            let result = std::mem::transmute::<_, &F>(&())(&ctx, info, userdata);
            ::std::mem::forget(ctx);
            result
        }

        let o = unsafe {
            ffi::pa_context_get_server_info(self.raw_mut(), Some(wrapped::<CB>), userdata)
        };
        if o.is_null() {
            return Err(ErrorCode::from_error_code(self.errno()));
        }
        Ok(unsafe { operation::from_raw_ptr(o) })
    }

    pub fn get_sink_info_by_name<CB>(&self,
                                     name: &CStr,
                                     _: CB,
                                     userdata: *mut c_void)
                                     -> Result<Operation>
        where CB: Fn(&Context, *const SinkInfo, i32, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(c: *mut ffi::pa_context,
                                        info: *const ffi::pa_sink_info,
                                        eol: c_int,
                                        userdata: *mut c_void)
            where F: Fn(&Context, *const SinkInfo, i32, *mut c_void)
        {
            let ctx = context::from_raw_ptr(c);
            let result = ::std::mem::transmute::<_, &F>(&())(&ctx, info, eol, userdata);
            ::std::mem::forget(ctx);
            result
        }

        let o = unsafe {
            ffi::pa_context_get_sink_info_by_name(self.raw_mut(),
                                                  name.as_ptr(),
                                                  Some(wrapped::<CB>),
                                                  userdata)
        };
        if o.is_null() {
            return Err(ErrorCode::from_error_code(self.errno()));
        }
        Ok(unsafe { operation::from_raw_ptr(o) })
    }

    pub fn get_sink_info_list<CB>(&self, _: CB, userdata: *mut c_void) -> Result<Operation>
        where CB: Fn(&Context, *const SinkInfo, i32, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(c: *mut ffi::pa_context,
                                        info: *const ffi::pa_sink_info,
                                        eol: c_int,
                                        userdata: *mut c_void)
            where F: Fn(&Context, *const SinkInfo, i32, *mut c_void)
        {
            let ctx = context::from_raw_ptr(c);
            let result = ::std::mem::transmute::<_, &F>(&())(&ctx, info, eol, userdata);
            ::std::mem::forget(ctx);
            result
        }

        let o = unsafe {
            ffi::pa_context_get_sink_info_list(self.raw_mut(), Some(wrapped::<CB>), userdata)
        };
        if o.is_null() {
            return Err(ErrorCode::from_error_code(self.errno()));
        }
        Ok(unsafe { operation::from_raw_ptr(o) })
    }

    pub fn get_sink_input_info<CB>(&self,
                                   idx: u32,
                                   _: CB,
                                   userdata: *mut c_void)
                                   -> Result<Operation>
        where CB: Fn(&Context, *const SinkInputInfo, i32, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(c: *mut ffi::pa_context,
                                        info: *const ffi::pa_sink_input_info,
                                        eol: c_int,
                                        userdata: *mut c_void)
            where F: Fn(&Context, *const SinkInputInfo, i32, *mut c_void)
        {
            let ctx = context::from_raw_ptr(c);
            let result = ::std::mem::transmute::<_, &F>(&())(&ctx, info, eol, userdata);
            std::mem::forget(ctx);
            result
        }

        let o = unsafe {
            ffi::pa_context_get_sink_input_info(self.raw_mut(), idx, Some(wrapped::<CB>), userdata)
        };
        if o.is_null() {
            return Err(ErrorCode::from_error_code(self.errno()));
        }
        Ok(unsafe { ::operation::from_raw_ptr(o) })
    }

    pub fn get_source_info_list<CB>(&self, _: CB, userdata: *mut c_void) -> Result<Operation>
        where CB: Fn(&Context, *const SourceInfo, i32, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(c: *mut ffi::pa_context,
                                        info: *const ffi::pa_source_info,
                                        eol: c_int,
                                        userdata: *mut c_void)
            where F: Fn(&Context, *const SourceInfo, i32, *mut c_void)
        {
            let ctx = context::from_raw_ptr(c);
            let result = ::std::mem::transmute::<_, &F>(&())(&ctx, info, eol, userdata);
            ::std::mem::forget(ctx);
            result
        }

        let o = unsafe {
            ffi::pa_context_get_source_info_list(self.raw_mut(), Some(wrapped::<CB>), userdata)
        };
        if o.is_null() {
            return Err(ErrorCode::from_error_code(self.errno()));
        }
        Ok(unsafe { ::operation::from_raw_ptr(o) })
    }

    pub fn set_sink_input_volume<CB>(&self,
                                     idx: u32,
                                     volume: &CVolume,
                                     _: CB,
                                     userdata: *mut c_void)
                                     -> Result<Operation>
        where CB: Fn(&Context, i32, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(c: *mut ffi::pa_context,
                                        success: c_int,
                                        userdata: *mut c_void)
            where F: Fn(&Context, i32, *mut c_void)
        {
            let ctx = context::from_raw_ptr(c);
            let result = ::std::mem::transmute::<_, &F>(&())(&ctx, success, userdata);
            std::mem::forget(ctx);
            result
        }

        let o = unsafe {
            ffi::pa_context_set_sink_input_volume(self.raw_mut(),
                                                  idx,
                                                  volume,
                                                  Some(wrapped::<CB>),
                                                  userdata)
        };
        if o.is_null() {
            return Err(ErrorCode::from_error_code(self.errno()));
        }
        Ok(unsafe { operation::from_raw_ptr(o) })
    }

    pub fn subscribe<CB>(&self,
                         m: SubscriptionMask,
                         _: CB,
                         userdata: *mut c_void)
                         -> Result<Operation>
        where CB: Fn(&Context, i32, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(c: *mut ffi::pa_context,
                                        success: c_int,
                                        userdata: *mut c_void)
            where F: Fn(&Context, i32, *mut c_void)
        {
            let ctx = context::from_raw_ptr(c);
            let result = ::std::mem::transmute::<_, &F>(&())(&ctx, success, userdata);
            std::mem::forget(ctx);
            result
        }

        let o = unsafe {
            ffi::pa_context_subscribe(self.raw_mut(), m.into(), Some(wrapped::<CB>), userdata)
        };
        if o.is_null() {
            return Err(ErrorCode::from_error_code(self.errno()));
        }
        Ok(unsafe { operation::from_raw_ptr(o) })
    }

    pub fn clear_subscribe_callback(&self) {
        unsafe {
            ffi::pa_context_set_subscribe_callback(self.raw_mut(), None, ptr::null_mut());
        }
    }

    pub fn set_subscribe_callback<CB>(&self, _: CB, userdata: *mut c_void)
        where CB: Fn(&Context, SubscriptionEvent, u32, *mut c_void)
    {
        debug_assert_eq!(::std::mem::size_of::<CB>(), 0);

        unsafe extern "C" fn wrapped<F>(c: *mut ffi::pa_context,
                                        t: ffi::pa_subscription_event_type_t,
                                        idx: u32,
                                        userdata: *mut c_void)
            where F: Fn(&Context, SubscriptionEvent, u32, *mut c_void)
        {
            let ctx = context::from_raw_ptr(c);
            let event = SubscriptionEvent::try_from(t)
            .expect("pa_context_subscribe_cb_t passed invalid pa_subscription_event_type_t");
            let result = ::std::mem::transmute::<_, &F>(&())(&ctx, event, idx, userdata);
            ::std::mem::forget(ctx);
            result
        }

        unsafe {
            ffi::pa_context_set_subscribe_callback(self.raw_mut(), Some(wrapped::<CB>), userdata);
        }
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        unsafe { Context(ffi::pa_context_ref(self.raw_mut())) }
    }
}

impl ::std::default::Default for Context {
    fn default() -> Self {
        Context(::std::ptr::null_mut())
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.is_null() {
            unsafe {
                ffi::pa_context_unref(self.raw_mut());
            }
        }
    }
}

#[doc(hidden)]
pub unsafe fn from_raw_ptr(ptr: *mut ffi::pa_context) -> Context {
    Context(ptr)
}