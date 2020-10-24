//
//  EsperantoDeviceTests.swift
//  EsperantoDeviceTests
//
//  Created by Alastair Coote on 9/1/20.
//  Copyright © 2020 Alastair Coote. All rights reserved.
//

import XCTest
import Esperanto

class EsperantoDeviceTests: XCTestCase {

    override func setUpWithError() throws {
        // Put setup code here. This method is called before the invocation of each test method in the class.

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false

        // In UI tests it’s important to set the initial state - such as interface orientation - required for your tests before they run. The setUp method is a good place to do this.
    }

    override func tearDownWithError() throws {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }

    func testExample() throws {
        let ctx = JSContext()
        ctx.evaluate(script: "const result = []; const print = function(p) {result.push(p)};")

        let jsPath = Bundle(for: EsperantoDeviceTests.self).url(forResource: "jsperf", withExtension: "js")!
        let js = try! String(contentsOf: jsPath)

        var compiled:UnsafeMutablePointer<CompiledCode>? = nil

//        measure {
            compiled = ctx.compile(script: js)
//        }

        let ctx2 = JSContext()
        measure {
            ctx2.evaluate(compiledCode: compiled!)
        }


//        BenchmarkSuite.RunSuites({
//            NotifyResult: PrintResult,
//            NotifyError: PrintError,
//            NotifyScore: PrintScore,
//        });

    }

//    func testLaunchPerformance() throws {
//        if #available(macOS 10.15, iOS 13.0, tvOS 13.0, *) {
//            // This measures how long it takes to launch your application.
//            measure(metrics: [XCTApplicationLaunchMetric()]) {
//                XCUIApplication().launch()
//            }
//        }
//    }
}
