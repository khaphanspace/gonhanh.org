import SwiftUI

struct AboutView: View {
    var body: some View {
        VStack(spacing: 0) {
            content
            Divider()
            footer
        }
        .frame(width: 360)
    }

    // MARK: - Header
    private var content: some View {
        VStack(spacing: 16) {
            Spacer()

            Image(nsImage: AppMetadata.logo)
                .resizable()
                .frame(width: 80, height: 80)

            VStack(spacing: 4) {
                Text(AppMetadata.name)
                    .font(.title2.bold())
                Text(AppMetadata.tagline)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            // Version badge
            HStack(spacing: 16) {
                VStack(spacing: 2) {
                    Text("Phiên bản")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                    Text(AppMetadata.version)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                VStack(spacing: 2) {
                    Text("Build")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                    Text(AppMetadata.buildNumber)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }

            Spacer()

            // Info
            VStack(spacing: 8) {
                infoRow("Tác giả", AppMetadata.author)
                infoRow("Công nghệ", AppMetadata.techStack)
                infoRow("Giấy phép", AppMetadata.license)
            }

            Spacer()

            // Links
            HStack(spacing: 20) {
                Link(destination: URL(string: AppMetadata.website)!) {
                    Label("Website", systemImage: "globe")
                }
                Link(destination: URL(string: AppMetadata.repository)!) {
                    Label("GitHub", systemImage: "chevron.left.forwardslash.chevron.right")
                }
            }
            .font(.callout)

            Spacer()
        }
        .padding(.horizontal, 28)
        .padding(.vertical, 20)
    }

    private func infoRow(_ label: String, _ value: String) -> some View {
        HStack {
            Text(label)
                .foregroundStyle(.secondary)
            Spacer()
            Text(value)
        }
        .font(.callout)
    }

    private var footer: some View {
        Text(AppMetadata.copyright)
            .font(.caption2)
            .foregroundStyle(.tertiary)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 12)
    }
}

#Preview {
    AboutView()
}
