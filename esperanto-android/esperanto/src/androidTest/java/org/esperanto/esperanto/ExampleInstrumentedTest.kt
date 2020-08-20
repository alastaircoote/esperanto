package org.esperanto.esperanto

import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.runners.AndroidJUnit4

import org.junit.Test
import org.junit.runner.RunWith

import org.junit.Assert.*

/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */

@RunWith(AndroidJUnit4::class)
class ExampleInstrumentedTest {
    class NumberGeneratorProxy(private val jsvalue:JSValue) : NumberGenerator {
        override fun generate(): Double {
            return jsvalue.getProperty("generate").call(jsvalue).asNumber()
        }
    }

    interface NumberGenerator {
        fun generate() : Double
    }

//    @Test
//    fun does_it_work() {
//        val ctx = JSContext()
//        val jsvalue = ctx.evaluate("""
//            class JSNumberGenerator {
//                constructor() {
//                    this.currentNumber = 0;
//                }
//                generate() {
//                    this.currentNumber++;
//                    return this.currentNumber;
//                }
//            }
//
//            new JSNumberGenerator();
//        """.trimIndent())
//
//        val generator:NumberGenerator = jsvalue.cast()
//
//        val number = generator.generate()
//        assertEquals(1.0,number,0.0)
//        val number2 = generator.generate()
//        assertEquals(2.0,number2,0.0)
//    }

    @Test
    fun do_strings_work() {
        val ctx = JSContext()
        val jsvalue = ctx.evaluate("'hello'")
        val str = jsvalue.asString()
        assertEquals(str, "hello")
    }
}