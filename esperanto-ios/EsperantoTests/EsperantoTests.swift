//
//  EsperantoTests.swift
//  EsperantoTests
//
//  Created by Alastair Coote on 8/4/20.
//  Copyright © 2020 Alastair Coote. All rights reserved.
//

import XCTest
@testable import Esperanto

protocol NumberGenerator : FromJSValue {
    func generate() -> Double
}

class EsperantoTests: XCTestCase {

//
//
//    func testNumberGenerator() {
//        let ctx = JSContext()
//        let val = ctx.evaluate(script: """
//            class JSNumberGenerator {
//                constructor() {
//                    this.currentNumber = 0;
//                }
//
//                generate() {
//                    this.currentNumber++;
//                    return this.currentNumber;
//                }
//            }
//
//            new JSNumberGenerator()
//        """)
//        // Cast to our native proxy
//        let generator:NumberGenerator = val.cast()
//
//        // Now use it as if it's a native class
//        let number = generator.generate()
//        assert(number == 1)
//        let number2 = generator.generate()
//        assert(number2 == 2)
//    }

    func testCompilingCode() {
        let ctx = JSContext()
        let compiled = ctx.compile(script: "var hello = 123456")
        let ctx2 = JSContext()
        ctx2.evaluate(compiledCode: compiled)
        let val = ctx2.evaluate(script: "hello")
        assert(val.toNumber() == 123456)
    }

}
