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

    override func setUpWithError() throws {
        // Put setup code here. This method is called before the invocation of each test method in the class.
    }

    override func tearDownWithError() throws {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }

    func testExample() throws {
        // This is an example of a functional test case.
        // Use XCTAssert and related functions to verify your tests produce the correct results.
    }

    func testPerformanceExample() throws {
        // This is an example of a performance test case.
        self.measure {
            // Put the code you want to measure the time of here.
        }
    }

    func testStr() {
        let ctx = JSContext()
        let val = ctx.evaluate(script: """
let number = 0;
function generate() {
    number++;
    return number;
}
generate
""")
        let number = val.call().toNumber()
        assert(number == 1)
        let number2 = val.call().toNumber()
        assert(number2 == 2)
    }

}
