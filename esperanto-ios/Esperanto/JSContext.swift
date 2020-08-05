//
//  JSContext.swift
//  Esperanto
//
//  Created by Alastair Coote on 8/4/20.
//  Copyright © 2020 Alastair Coote. All rights reserved.
//

import Foundation
import Esperanto.Esperanto_Private

public class JSContext {
    let ctx: OpaquePointer
    public init() {
        self.ctx = jscontext_new()
    }

    public func evaluate(script: String) -> JSValue {
        let val_ptr = jscontext_evaluate(self.ctx, script)
        return JSValue(ptr: val_ptr!)
    }

    deinit {
        jscontext_free(self.ctx)
    }
}
