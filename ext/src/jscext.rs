// jscext.rs
//
// Copyright (C) 2025 ZaynChen
//
// This file is part of LightDM WebKit Greeter
//
// LightDM WebKit Greeter is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// LightDM WebKit Greeter is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use gtk::glib::{
    ffi::{GPtrArray, gpointer},
    object::IsA,
    translate::*,
    types::StaticType,
};
use jsc::{
    Class, Context, Value,
    ffi::{
        JSCClass, JSCValue, jsc_class_add_property, jsc_context_register_class,
        jsc_value_new_object,
    },
};

use std::ptr::null_mut;

pub trait JSCValueExtManual: IsA<Value> + 'static {
    #[doc(alias = "jsc_value_new_object")]
    fn new_object(context: &Context, instance: Option<Value>, jsc_class: Option<&Class>) -> Value {
        let jsc_class = match jsc_class {
            Some(cls) => cls.to_glib_none().0,
            None => null_mut::<JSCClass>(),
        };

        let instance = match instance {
            Some(ins) => ins.to_glib_none().0,
            None => null_mut::<JSCValue>(),
        };

        unsafe {
            from_glib_none(jsc_value_new_object(
                context.to_glib_none().0,
                instance as gpointer,
                jsc_class,
            ))
        }
    }

    #[doc(alias = "jsc_value_new_function_variadic")]
    fn new_function_variadic<F>(context: &Context, name: Option<&str>, callback: F) -> Value
    where
        F: Fn(&[Value]) -> Option<Value> + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            params: *mut GPtrArray,
            user_data: gpointer,
        ) -> Option<Value>
        where
            F: Fn(&[Value]) -> Option<Value>,
        {
            unsafe {
                let f: &F = &*(user_data as *const F);
                f(&Value::from_glib_none_as_vec(params))
            }
        }

        unsafe extern "C" fn destroy_closure<F>(user_data: gpointer)
        where
            F: Fn(&[Value]) -> Option<Value>,
        {
            // destroy
            unsafe {
                let _ = Box::<F>::from_raw(user_data as *mut _);
            }
        }

        unsafe {
            let callback = Box::into_raw(Box::new(callback));
            from_glib_none(jsc::ffi::jsc_value_new_function_variadic(
                context.to_glib_none().0,
                name.to_glib_none().0,
                Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(
                    trampoline::<F> as *const (),
                )),
                callback as gpointer,
                Some(destroy_closure::<F>),
                Value::static_type().into_glib(),
            ))
        }
    }

    fn to_vec(&self) -> Vec<Value> {
        let this = self.as_ref();
        if !this.is_array() {
            panic!("jscvalue is not an array");
        }

        let length = this
            .object_get_property("length")
            .expect("object does not has property `length`")
            .to_int32() as u32;
        let mut array = Vec::with_capacity(length as usize);
        for i in 0..length {
            array.push(this.object_get_property_at_index(i).unwrap());
        }

        array
    }
}
impl<O: IsA<Value>> JSCValueExtManual for O {}

pub trait JSCContextExtManual: IsA<Context> + 'static {
    #[doc(alias = "jsc_context_register_class")]
    fn register_class(
        &self,
        name: &str,
        parent_class: Option<&Class>,
        // vtable: *mut JSCClassVTable,
    ) -> Option<Class> {
        let vtable = null_mut();
        unsafe {
            from_glib_none(jsc_context_register_class(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                parent_class.to_glib_none().0,
                vtable,
                None,
            ))
        }
    }
}
impl<O: IsA<Context>> JSCContextExtManual for O {}

pub trait JSCClassExtManual: IsA<Class> + 'static {
    #[doc(alias = "jsc_class_add_constructor_variadic")]
    fn add_constructor_variadic<F>(&self, name: Option<&str>, callback: F) -> Option<Value>
    where
        F: Fn(&[Value]) -> Option<Value> + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            params: *mut GPtrArray,
            user_data: gpointer,
        ) -> Option<Value>
        where
            F: Fn(&[Value]) -> Option<Value>,
        {
            unsafe {
                let f: &F = &*(user_data as *const F);
                f(&Value::from_glib_none_as_vec(params))
            }
        }

        unsafe extern "C" fn destroy_closure<F>(user_data: gpointer)
        where
            F: Fn(&[Value]) -> Option<Value>,
        {
            // destroy
            unsafe {
                let _ = Box::<F>::from_raw(user_data as *mut _);
            }
        }

        unsafe {
            let callback = Box::into_raw(Box::new(callback));
            from_glib_full(jsc::ffi::jsc_class_add_constructor_variadic(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(
                    trampoline::<F> as *const (),
                )),
                callback as gpointer,
                Some(destroy_closure::<F>),
                Value::static_type().into_glib(),
            ))
        }
    }

    #[doc(alias = "jsc_class_add_method_variadic")]
    fn add_method_variadic<F>(&self, name: &str, callback: F)
    where
        F: Fn(&Value, &[Value]) -> Option<Value> + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            this: *mut JSCValue,
            params: *mut GPtrArray,
            user_data: gpointer,
        ) -> Option<Value>
        where
            F: Fn(&Value, &[Value]) -> Option<Value>,
        {
            unsafe {
                let f: &F = &*(user_data as *const F);
                f(&from_glib_none(this), &Value::from_glib_none_as_vec(params))
            }
        }

        unsafe extern "C" fn destroy_closure<F>(user_data: gpointer)
        where
            F: Fn(&Value, &[Value]) -> Option<Value>,
        {
            // destroy
            unsafe {
                let _ = Box::<F>::from_raw(user_data as *mut _);
            }
        }

        unsafe {
            let callback = Box::into_raw(Box::new(callback));
            jsc::ffi::jsc_class_add_method_variadic(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(
                    trampoline::<F> as *const (),
                )),
                callback as gpointer,
                Some(destroy_closure::<F>),
                Value::static_type().into_glib(),
            );
        }
    }

    fn add_property<F>(&self, name: &str, has_getter: bool, has_setter: bool, accessor: F)
    where
        F: Fn(&Value, Option<Value>) -> Option<Value> + 'static,
    {
        let getter_trampoline = if has_getter {
            unsafe extern "C" fn getter_trampoline<F>(
                this: *mut JSCValue,
                user_data: gpointer,
            ) -> Value
            where
                F: Fn(&Value, Option<Value>) -> Option<Value>,
            {
                unsafe {
                    let f: &F = &*(user_data as *const F);
                    f(&from_glib_none(this), None).unwrap()
                }
            }
            unsafe {
                Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(
                    getter_trampoline::<F> as *const (),
                ))
            }
        } else {
            None
        };

        let setter_trampoline = if has_setter {
            unsafe extern "C" fn setter_trampoline<F>(
                this: *mut JSCValue,
                value: *mut JSCValue,
                user_data: gpointer,
            ) where
                F: Fn(&Value, Option<Value>) -> Option<Value>,
            {
                unsafe {
                    let f: &F = &*(user_data as *const F);
                    f(&from_glib_none(this), Some(from_glib_none(value)));
                }
            }
            unsafe {
                Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(
                    setter_trampoline::<F> as *const (),
                ))
            }
        } else {
            None
        };

        unsafe extern "C" fn destroy_closure<F>(user_data: gpointer)
        where
            F: Fn(&Value, Option<Value>) -> Option<Value>,
        {
            // destroy
            unsafe {
                let _ = Box::from_raw(user_data as *mut F);
            }
        }

        unsafe {
            let user_data = Box::into_raw(Box::new(accessor));
            jsc_class_add_property(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                Value::static_type().into_glib(),
                getter_trampoline,
                setter_trampoline,
                user_data as gpointer,
                Some(destroy_closure::<F>),
            )
        }
    }
}
impl<O: IsA<Class>> JSCClassExtManual for O {}
