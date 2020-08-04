//
//  JSValue.swift
//  Esperanto
//
//  Created by Alastair Coote on 8/4/20.
//  Copyright © 2020 Alastair Coote. All rights reserved.
//

import Foundation
import Esperanto.Esperanto_Private

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

    func toNumber() -> Double {
        jsvalue_as_number(self.value_ptr)
    }

    deinit {
        jsvalue_free(value_ptr)
    }
}
