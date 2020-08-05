//
//  NumberGenerator.swift
//  Esperanto
//
//  Created by Alastair Coote on 8/5/20.
//  Copyright © 2020 Alastair Coote. All rights reserved.
//

import Foundation

protocol NumberGenerator : FromJSValue {
    func generate() -> Double
}
