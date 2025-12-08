import SwiftUI

struct AboutView: View {
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            infoSection
            Divider()
            linksSection
            Divider()
            footer
        }
        .frame(width: 360)
        .background(colorScheme == .dark ? Color.black.opacity(0.2) : Color.white)
    }

    // MARK: - Header
    private var header: some View {
        VStack(spacing: 8) {
            Image(nsImage: AppMetadata.logo)
                .resizable()
                .frame(width: 80, height: 80)
                .shadow(color: .black.opacity(0.1), radius: 4, y: 2)

            Text(AppMetadata.name)
                .font(.system(size: 24, weight: .bold, design: .rounded))

            Text(AppMetadata.tagline)
                .font(.subheadline)
                .foregroundStyle(.secondary)

            // Version badge
            HStack(spacing: 12) {
                versionBadge(label: "Version", value: AppMetadata.version)
                versionBadge(label: "Build", value: AppMetadata.buildNumber)
            }
            .padding(.top, 4)
        }
        .padding(.vertical, 24)
        .padding(.horizontal, 32)
    }

    private func versionBadge(label: String, value: String) -> some View {
        HStack(spacing: 4) {
            Text(label)
                .font(.caption2)
                .foregroundStyle(.tertiary)
            Text(value)
                .font(.caption)
                .fontWeight(.medium)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(colorScheme == .dark ? Color.white.opacity(0.08) : Color.black.opacity(0.04))
        )
    }

    // MARK: - Info Section
    private var infoSection: some View {
        VStack(spacing: 12) {
            infoRow(icon: "person.fill", title: "Developer", value: AppMetadata.author)
            infoRow(icon: "envelope.fill", title: "Contact", value: AppMetadata.authorEmail, isLink: true, url: "mailto:\(AppMetadata.authorEmail)")
            infoRow(icon: "hammer.fill", title: "Built with", value: AppMetadata.techStack)
            infoRow(icon: "doc.text.fill", title: "License", value: AppMetadata.license)
        }
        .padding(.vertical, 16)
        .padding(.horizontal, 24)
    }

    private func infoRow(icon: String, title: String, value: String, isLink: Bool = false, url: String? = nil) -> some View {
        HStack(spacing: 12) {
            Image(systemName: icon)
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
                .frame(width: 20)

            Text(title)
                .font(.callout)
                .foregroundStyle(.secondary)
                .frame(width: 80, alignment: .leading)

            if isLink, let urlString = url, let linkURL = URL(string: urlString) {
                Link(value, destination: linkURL)
                    .font(.callout)
                    .fontWeight(.medium)
            } else {
                Text(value)
                    .font(.callout)
                    .fontWeight(.medium)
                    .foregroundStyle(.primary)
            }

            Spacer()
        }
    }

    // MARK: - Links Section
    private var linksSection: some View {
        HStack(spacing: 16) {
            linkButton(icon: "globe", title: "Website", url: AppMetadata.website)
            linkButton(icon: "chevron.left.forwardslash.chevron.right", title: "GitHub", url: AppMetadata.repository)
            linkButton(icon: "exclamationmark.bubble.fill", title: "Issues", url: AppMetadata.issuesURL)
            linkButton(icon: "link", title: "LinkedIn", url: AppMetadata.authorLinkedin)
        }
        .padding(.vertical, 16)
        .padding(.horizontal, 24)
    }

    private func linkButton(icon: String, title: String, url: String) -> some View {
        Link(destination: URL(string: url)!) {
            VStack(spacing: 6) {
                Image(systemName: icon)
                    .font(.system(size: 16))
                Text(title)
                    .font(.caption2)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 8)
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(colorScheme == .dark ? Color.white.opacity(0.06) : Color.black.opacity(0.03))
            )
        }
        .buttonStyle(.plain)
        .foregroundStyle(.secondary)
        .onHover { hovering in
            if hovering {
                NSCursor.pointingHand.push()
            } else {
                NSCursor.pop()
            }
        }
    }

    // MARK: - Footer
    private var footer: some View {
        Text(AppMetadata.copyright)
            .font(.caption2)
            .foregroundStyle(.tertiary)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 12)
            .background(colorScheme == .dark ? Color.white.opacity(0.02) : Color.black.opacity(0.02))
    }
}

#Preview {
    AboutView()
}
