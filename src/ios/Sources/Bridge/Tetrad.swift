import TetradCore
import Foundation

public enum TetradBridge {
    public static func tetradInit() -> String {
        guard let cString = tetrad_init() else {
            return "Failed to init Tetrad"
        }
        let result = String(cString: cString)
        return result
    }
}
