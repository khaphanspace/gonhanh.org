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
        .background(Color.adaptiveSurface(colorScheme))
    }

    // MARK: - Header
    private var header: some View {
        VStack(spacing: DesignTokens.Spacing.space2) {
            Image(nsImage: AppMetadata.logo)
                .resizable()
                .frame(width: 80, height: 80)
                .shadow(color: .black.opacity(0.1), radius: 4, y: 2)

            Text(AppMetadata.name)
                .font(.system(size: DesignTokens.Typography.text2xl, weight: .bold, design: .rounded))

            Text(AppMetadata.tagline)
                .font(.subheadline)
                .foregroundStyle(Color.inkSecondary)

            // Version badge
            HStack(spacing: DesignTokens.Spacing.space3) {
                versionBadge(label: "Version", value: AppMetadata.version)
                versionBadge(label: "Build", value: AppMetadata.buildNumber)
            }
            .padding(.top, DesignTokens.Spacing.space1)
        }
        .padding(.vertical, DesignTokens.Spacing.space6)
        .padding(.horizontal, DesignTokens.Spacing.space8)
    }

    private func versionBadge(label: String, value: String) -> some View {
        HStack(spacing: DesignTokens.Spacing.space1) {
            Text(label)
                .font(.caption2)
                .foregroundStyle(Color.inkTertiary)
            Text(value)
                .font(.caption)
                .fontWeight(.medium)
                .foregroundStyle(Color.inkSecondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background(
            RoundedRectangle(cornerRadius: DesignTokens.Radius.sm)
                .fill(Color.adaptiveSurfaceSecondary(colorScheme))
        )
    }

    // MARK: - Info Section
    private var infoSection: some View {
        VStack(spacing: DesignTokens.Spacing.space3) {
            infoRow(icon: "person.fill", title: "Developer", value: AppMetadata.author)
            infoRow(icon: "envelope.fill", title: "Contact", value: AppMetadata.authorEmail, isLink: true, url: "mailto:\(AppMetadata.authorEmail)")
            infoRow(icon: "hammer.fill", title: "Built with", value: AppMetadata.techStack)
            infoRow(icon: "doc.text.fill", title: "License", value: AppMetadata.license)
        }
        .padding(.vertical, DesignTokens.Spacing.space4)
        .padding(.horizontal, DesignTokens.Spacing.space6)
    }

    private func infoRow(icon: String, title: String, value: String, isLink: Bool = false, url: String? = nil) -> some View {
        HStack(spacing: DesignTokens.Spacing.space3) {
            Image(systemName: icon)
                .font(.system(size: DesignTokens.Typography.textXs))
                .foregroundStyle(Color.inkSecondary)
                .frame(width: 20)

            Text(title)
                .font(.callout)
                .foregroundStyle(Color.inkSecondary)
                .frame(width: 80, alignment: .leading)

            if isLink, let urlString = url, let linkURL = URL(string: urlString) {
                Link(value, destination: linkURL)
                    .font(.callout)
                    .fontWeight(.medium)
                    .foregroundStyle(Color.accent)
            } else {
                Text(value)
                    .font(.callout)
                    .fontWeight(.medium)
                    .foregroundStyle(Color.adaptiveInk(colorScheme))
            }

            Spacer()
        }
    }

    // MARK: - Links Section
    private var linksSection: some View {
        HStack(spacing: DesignTokens.Spacing.space4) {
            linkButton(icon: "globe", title: "Website", url: AppMetadata.website)
            linkButton(icon: "chevron.left.forwardslash.chevron.right", title: "GitHub", url: AppMetadata.repository)
            linkButton(icon: "exclamationmark.bubble.fill", title: "Issues", url: AppMetadata.issuesURL)
            linkButton(icon: "link", title: "LinkedIn", url: AppMetadata.authorLinkedin)
        }
        .padding(.vertical, DesignTokens.Spacing.space4)
        .padding(.horizontal, DesignTokens.Spacing.space6)
    }

    private func linkButton(icon: String, title: String, url: String) -> some View {
        Link(destination: URL(string: url)!) {
            VStack(spacing: DesignTokens.Spacing.space1_5) {
                Image(systemName: icon)
                    .font(.system(size: DesignTokens.Typography.textBase))
                Text(title)
                    .font(.caption2)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, DesignTokens.Spacing.space2)
            .background(
                RoundedRectangle(cornerRadius: DesignTokens.Radius.md)
                    .fill(Color.adaptiveSurfaceSecondary(colorScheme))
            )
        }
        .buttonStyle(.plain)
        .foregroundStyle(Color.inkSecondary)
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
            .foregroundStyle(Color.inkTertiary)
            .frame(maxWidth: .infinity)
            .padding(.vertical, DesignTokens.Spacing.space3)
            .background(Color.adaptiveSurfaceSecondary(colorScheme))
    }
}

#Preview {
    AboutView()
}
