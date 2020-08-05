//
//  EsperantoTests.swift
//  EsperantoTests
//
//  Created by Alastair Coote on 8/4/20.
//  Copyright © 2020 Alastair Coote. All rights reserved.
//

import XCTest
@testable import Esperanto



class EsperantoTests: XCTestCase {

    func testNumberGenerator() {
        let ctx = JSContext()
        let val = ctx.evaluate(script: """
            class JSNumberGenerator {
                constructor() {
                    this.currentNumber = 0;
                }

                generate() {
                    this.currentNumber++;
                    return this.currentNumber;
                }
            }

            new JSNumberGenerator()
""")
        
        let generator:NumberGenerator = val.cast()
        let number = generator.generate()
        assert(number == 1)
        let number2 = generator.generate()
        assert(number2 == 2)
    }

}
