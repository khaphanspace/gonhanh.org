#!/usr/bin/env node

/**
 * Generate CONTRIBUTORS.md automatically
 * Fetches: Sponsors, Code Contributors, Issue Reporters, Commenters
 */

const REPO_OWNER = 'khaphanspace';
const REPO_NAME = 'gonhanh.org';

const GITHUB_TOKEN = process.env.GITHUB_TOKEN;

if (!GITHUB_TOKEN) {
  console.error('Error: GITHUB_TOKEN is required');
  process.exit(1);
}

const headers = {
  Authorization: `Bearer ${GITHUB_TOKEN}`,
  'Content-Type': 'application/json',
  'User-Agent': 'GoNhanh-Contributors-Bot',
};

// Bots to exclude
const EXCLUDED_USERS = new Set([
  'github-actions[bot]',
  'dependabot[bot]',
  'renovate[bot]',
  'codecov[bot]',
  'vercel[bot]',
  'netlify[bot]',
]);

/**
 * Fetch sponsors via GraphQL
 */
async function fetchSponsors() {
  const query = `
    query {
      user(login: "${REPO_OWNER}") {
        sponsorshipsAsMaintainer(first: 100, activeOnly: true) {
          nodes {
            tier {
              name
              monthlyPriceInDollars
            }
            sponsorEntity {
              ... on User {
                login
                avatarUrl
                url
              }
              ... on Organization {
                login
                avatarUrl
                url
              }
            }
          }
        }
      }
    }
  `;

  try {
    const res = await fetch('https://api.github.com/graphql', {
      method: 'POST',
      headers,
      body: JSON.stringify({ query }),
    });

    const data = await res.json();
    const sponsors = data?.data?.user?.sponsorshipsAsMaintainer?.nodes || [];

    // Categorize by tier
    const diamond = [];
    const gold = [];
    const silver = [];
    const backers = [];

    for (const s of sponsors) {
      const price = s.tier?.monthlyPriceInDollars || 0;
      const sponsor = {
        login: s.sponsorEntity.login,
        avatar: s.sponsorEntity.avatarUrl,
        url: s.sponsorEntity.url,
      };

      if (price >= 50) diamond.push(sponsor);
      else if (price >= 20) gold.push(sponsor);
      else if (price >= 5) silver.push(sponsor);
      else backers.push(sponsor);
    }

    return { diamond, gold, silver, backers };
  } catch (err) {
    console.error('Error fetching sponsors:', err.message);
    return { diamond: [], gold: [], silver: [], backers: [] };
  }
}

/**
 * Fetch code contributors via REST API
 */
async function fetchContributors() {
  try {
    const res = await fetch(
      `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/contributors?per_page=100`,
      { headers }
    );
    const data = await res.json();

    return data
      .filter((u) => !EXCLUDED_USERS.has(u.login) && u.type !== 'Bot')
      .map((u) => ({
        login: u.login,
        avatar: u.avatar_url,
        url: u.html_url,
        contributions: u.contributions,
      }));
  } catch (err) {
    console.error('Error fetching contributors:', err.message);
    return [];
  }
}

/**
 * Fetch all pages from a paginated GitHub API endpoint
 */
async function fetchAllPages(url) {
  const results = [];
  let nextUrl = url;

  while (nextUrl) {
    const res = await fetch(nextUrl, { headers });
    const data = await res.json();
    if (Array.isArray(data)) {
      results.push(...data);
    }

    // Parse Link header for next page
    const linkHeader = res.headers.get('Link');
    nextUrl = null;
    if (linkHeader) {
      const match = linkHeader.match(/<([^>]+)>;\s*rel="next"/);
      if (match) nextUrl = match[1];
    }
  }

  return results;
}

/**
 * Fetch community contributors - count all interactions as 1 each:
 * - Opening an issue = 1
 * - Commenting on issue/PR = 1
 * - Discussion post/comment = 1
 */
async function fetchCommunityContributors() {
  try {
    const baseUrl = `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}`;
    const [issues, issueComments, prComments] = await Promise.all([
      fetchAllPages(`${baseUrl}/issues?state=all&per_page=100`),
      fetchAllPages(`${baseUrl}/issues/comments?per_page=100`),
      fetchAllPages(`${baseUrl}/pulls/comments?per_page=100`),
    ]);

    // Fetch discussions via GraphQL
    let discussions = [];
    try {
      const query = `
        query {
          repository(owner: "${REPO_OWNER}", name: "${REPO_NAME}") {
            discussions(first: 100) {
              nodes {
                author { login avatarUrl url }
                comments(first: 100) {
                  nodes {
                    author { login avatarUrl url }
                  }
                }
              }
            }
          }
        }
      `;
      const res = await fetch('https://api.github.com/graphql', {
        method: 'POST',
        headers,
        body: JSON.stringify({ query }),
      });
      const data = await res.json();
      discussions = data?.data?.repository?.discussions?.nodes || [];
    } catch (e) {
      console.log('Discussions not available or error:', e.message);
    }

    const users = new Map();

    // Helper to add/increment user
    const addUser = (user) => {
      if (!user || EXCLUDED_USERS.has(user.login)) return;
      const login = user.login;
      if (!users.has(login)) {
        users.set(login, {
          login,
          avatar: user.avatarUrl || user.avatar_url,
          url: user.html_url || user.url,
          count: 0,
        });
      }
      users.get(login).count++;
    };

    // Count issue opens (skip PRs)
    for (const issue of issues) {
      if (!issue.pull_request) addUser(issue.user);
    }

    // Count issue comments
    for (const comment of issueComments) {
      addUser(comment.user);
    }

    // Count PR comments
    for (const comment of prComments) {
      addUser(comment.user);
    }

    // Count discussion posts and comments
    for (const discussion of discussions) {
      addUser(discussion.author);
      for (const comment of discussion.comments?.nodes || []) {
        addUser(comment.author);
      }
    }

    return Array.from(users.values()).sort((a, b) => b.count - a.count);
  } catch (err) {
    console.error('Error fetching community contributors:', err.message);
    return [];
  }
}

/**
 * Generate user table with avatars and names
 * @param {Array} users - Array of user objects
 * @param {Object} opts - Options: size, perRow, showSub, badge
 */
function userTableHtml(users, { size = 50, perRow = 7, showSub = null, badge = '' } = {}) {
  if (users.length === 0) return '<p><em>Chưa có</em></p>';

  let html = '<table>\n';
  for (let i = 0; i < users.length; i += perRow) {
    const row = users.slice(i, i + perRow);
    html += '  <tr>\n';
    for (const u of row) {
      const subtitle = showSub && u[showSub] ? `<br/><sub>${u[showSub]} commits</sub>` : '';
      const badgeHtml = badge ? ` ${badge}` : '';
      html += `    <td align="center">
      <a href="${u.url}">
        <img src="${u.avatar}" width="${size}" style="border-radius:50%"/><br/>
        <b>${u.login}</b>${badgeHtml}
      </a>${subtitle}
    </td>\n`;
    }
    html += '  </tr>\n';
  }
  html += '</table>';
  return html;
}

/**
 * Generate full markdown
 */
function generateMarkdown(sponsors, contributors, community) {
  // Deduplicate: remove code contributors from community list
  const contributorLogins = new Set(contributors.map((c) => c.login));
  const filteredCommunity = community.filter(
    (u) => !contributorLogins.has(u.login)
  );

  const now = new Date().toLocaleDateString('vi-VN', { day: '2-digit', month: '2-digit', year: 'numeric' });

  return `<div align="center">

# 🌟 Cộng đồng Gõ Nhanh

**Gõ Nhanh được xây dựng bởi cộng đồng, cho cộng đồng.**

Mỗi đóng góp, dù lớn hay nhỏ, đều giúp người Việt gõ tiếng Việt tốt hơn mỗi ngày.

</div>

---

## 💖 Sponsors

Những người đã tin tưởng và ủng hộ dự án.

${
  sponsors.diamond.length > 0
    ? `### 💎 Diamond

${userTableHtml(sponsors.diamond, { size: 120, perRow: 5, badge: '💎' })}
`
    : ''
}
${
  sponsors.gold.length > 0
    ? `### 🥇 Gold

${userTableHtml(sponsors.gold, { size: 100, perRow: 6, badge: '🥇' })}
`
    : ''
}
${
  sponsors.silver.length > 0
    ? `### 🥈 Silver

${userTableHtml(sponsors.silver, { size: 80, perRow: 7, badge: '🥈' })}
`
    : ''
}
${
  sponsors.backers.length > 0
    ? `### 💜 Backers

${userTableHtml(sponsors.backers, { size: 50, perRow: 8, badge: '💜' })}
`
    : ''
}
${
  sponsors.diamond.length === 0 &&
  sponsors.gold.length === 0 &&
  sponsors.silver.length === 0 &&
  sponsors.backers.length === 0
    ? `<p align="center"><em>Chưa có sponsor nào. Hãy là người đầu tiên!</em></p>
`
    : ''
}
<p align="center">
  <a href="https://github.com/sponsors/${REPO_OWNER}">
    <img src="https://img.shields.io/badge/Trở_thành_Sponsor-💖-ea4aaa?style=for-the-badge" alt="Sponsor"/>
  </a>
</p>

---

## 💻 Code Contributors

Những người đã đóng góp code, biến ý tưởng thành hiện thực.

${userTableHtml(contributors, { size: 80, perRow: 7, showSub: 'contributions' })}

---

## 💬 Cộng đồng

Những người đã báo lỗi, góp ý, và thảo luận giúp định hình sản phẩm.

${userTableHtml(filteredCommunity, { size: 50, perRow: 6 })}

---

<div align="center">

### Bạn muốn đóng góp?

<a href="/CONTRIBUTING.md">📖 Hướng dẫn đóng góp</a> ·
<a href="https://github.com/${REPO_OWNER}/${REPO_NAME}/issues">🐛 Báo lỗi</a> ·
<a href="https://github.com/sponsors/${REPO_OWNER}">💖 Sponsor</a>

---

*Được cập nhật tự động · Lần cuối: ${now}*

</div>
`;
}

/**
 * Main
 */
async function main() {
  console.log('Fetching data from GitHub...');

  const [sponsors, contributors, community] = await Promise.all([
    fetchSponsors(),
    fetchContributors(),
    fetchCommunityContributors(),
  ]);

  const sponsorCount =
    sponsors.diamond.length +
    sponsors.gold.length +
    sponsors.silver.length +
    sponsors.backers.length;

  console.log(`Found:`);
  console.log(`  - ${sponsorCount} sponsors`);
  console.log(`  - ${contributors.length} code contributors`);
  console.log(`  - ${community.length} community contributors`);

  const markdown = generateMarkdown(sponsors, contributors, community);

  const fs = require('fs');
  const path = require('path');
  const outputPath = path.join(__dirname, '..', 'CONTRIBUTORS.md');

  fs.writeFileSync(outputPath, markdown);
  console.log(`\nGenerated: ${outputPath}`);
}

main().catch((err) => {
  console.error('Fatal error:', err);
  process.exit(1);
});
