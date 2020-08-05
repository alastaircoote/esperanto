//
//  JSValue.swift
//  Esperanto
//
//  Created by Alastair Coote on 8/4/20.
//  Copyright © 2020 Alastair Coote. All rights reserved.
//

import Foundation
import Esperanto.Esperanto_Private

protocol FromJSValue {
    init(jsValue: JSValue)
}

struct NumberGeneratorProxy : NumberGenerator, FromJSValue {

    let val: JSValue

    func generate() -> Double {
        return self.val.get(propertyByName: "generate").call(boundTo: self.val).toNumber()
    }

    init(jsValue: JSValue) {
        self.val = jsValue
    }


}

public class JSValue {

    let value_ptr: OpaquePointer

    init(ptr: OpaquePointer) {
        self.value_ptr = ptr
    }

    func toString() -> String {
        let str = jsvalue_as_string(self.value_ptr)!
        return String(cString: str)
    }

    func call() -> JSValue {
        let new_val_ptr = jsvalue_call(self.value_ptr)
        return JSValue(ptr: new_val_ptr!)
    }

    func call(boundTo: JSValue) -> JSValue {
        let new_val_ptr = jsvalue_call_bound(self.value_ptr, boundTo.value_ptr)
        return JSValue(ptr: new_val_ptr!)
    }

    func toNumber() -> Double {
        jsvalue_as_number(self.value_ptr)
    }



    func cast() -> String {
        return "wsdfs"
    }

    func get(propertyByName: String) -> JSValue {
        let ptr = jsvalue_get_property(self.value_ptr, propertyByName)!
        return JSValue(ptr: ptr)
    }

    deinit {
        jsvalue_free(value_ptr)
    }
}

extension JSValue {
    func cast() -> NumberGenerator {
        return NumberGeneratorProxy(jsValue: self)
    }
}
