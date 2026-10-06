import SwiftUI
import TetradBridge

@main
struct TetradApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    @State private var message = "Loading…"

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "sparkles")
                .font(.largeTitle)
                .foregroundStyle(.tint)

            Text("Tetrad")
                .font(.largeTitle.bold())

            Text(message)
                .font(.title2)
                .multilineTextAlignment(.center)
                .accessibilityIdentifier("rustMessage")
        }
        .padding(24)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .task {
            message = TetradBridge.tetradInit()
        }
    }
}
