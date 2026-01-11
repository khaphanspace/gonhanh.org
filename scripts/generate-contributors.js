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
 * Fetch issue creators
 */
async function fetchIssueCreators() {
  try {
    const res = await fetch(
      `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/issues?state=all&per_page=100`,
      { headers }
    );
    const data = await res.json();

    const users = new Map();
    for (const issue of data) {
      if (!issue.user || EXCLUDED_USERS.has(issue.user.login)) continue;
      if (issue.pull_request) continue; // Skip PRs

      const login = issue.user.login;
      if (!users.has(login)) {
        users.set(login, {
          login,
          avatar: issue.user.avatar_url,
          url: issue.user.html_url,
          count: 0,
        });
      }
      users.get(login).count++;
    }

    return Array.from(users.values()).sort((a, b) => b.count - a.count);
  } catch (err) {
    console.error('Error fetching issue creators:', err.message);
    return [];
  }
}

/**
 * Fetch commenters (issues + PRs)
 */
async function fetchCommenters() {
  try {
    const [issueComments, prComments] = await Promise.all([
      fetch(
        `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/issues/comments?per_page=100`,
        { headers }
      ).then((r) => r.json()),
      fetch(
        `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/pulls/comments?per_page=100`,
        { headers }
      ).then((r) => r.json()),
    ]);

    const users = new Map();
    const allComments = [...(issueComments || []), ...(prComments || [])];

    for (const comment of allComments) {
      if (!comment.user || EXCLUDED_USERS.has(comment.user.login)) continue;

      const login = comment.user.login;
      if (!users.has(login)) {
        users.set(login, {
          login,
          avatar: comment.user.avatar_url,
          url: comment.user.html_url,
          count: 0,
        });
      }
      users.get(login).count++;
    }

    return Array.from(users.values()).sort((a, b) => b.count - a.count);
  } catch (err) {
    console.error('Error fetching commenters:', err.message);
    return [];
  }
}

/**
 * Generate avatar HTML
 */
function avatarHtml(user, size = 50) {
  return `<a href="${user.url}"><img src="${user.avatar}" width="${size}" title="${user.login}"/></a>`;
}

/**
 * Generate table row for code contributors
 */
function contributorTableHtml(contributors, perRow = 7) {
  if (contributors.length === 0) return '<p><em>Chưa có</em></p>';

  let html = '<table>\n';
  for (let i = 0; i < contributors.length; i += perRow) {
    const row = contributors.slice(i, i + perRow);
    html += '  <tr>\n';
    for (const u of row) {
      html += `    <td align="center">
      <a href="${u.url}">
        <img src="${u.avatar}" width="80" style="border-radius:50%"/><br/>
        <b>${u.login}</b>
      </a>
      <br/>
      <sub>${u.contributions} commits</sub>
    </td>\n`;
    }
    html += '  </tr>\n';
  }
  html += '</table>';
  return html;
}

/**
 * Generate avatar grid
 */
function avatarGridHtml(users, size = 50) {
  if (users.length === 0) return '<p><em>Chưa có</em></p>';
  return users.map((u) => avatarHtml(u, size)).join('\n');
}

/**
 * Generate full markdown
 */
function generateMarkdown(sponsors, contributors, issueCreators, commenters) {
  // Deduplicate: remove contributors from issue creators and commenters
  const contributorLogins = new Set(contributors.map((c) => c.login));
  const filteredIssueCreators = issueCreators.filter(
    (u) => !contributorLogins.has(u.login)
  );
  const allKnownLogins = new Set([
    ...contributorLogins,
    ...filteredIssueCreators.map((u) => u.login),
  ]);
  const filteredCommenters = commenters.filter(
    (u) => !allKnownLogins.has(u.login)
  );

  const now = new Date().toLocaleDateString('vi-VN', { day: '2-digit', month: '2-digit', year: 'numeric' });

  return `<div align="center">

# 🌟 Cộng đồng Gõ Nhanh

**Gõ Nhanh được xây dựng bởi cộng đồng, cho cộng đồng.**

Mỗi đóng góp, dù lớn hay nhỏ, đều giúp người Việt gõ tiếng Việt tốt hơn mỗi ngày.

</div>

---

## 💖 Sponsors

Những người đã tin tưởng và hỗ trợ tài chính cho dự án.

${
  sponsors.diamond.length > 0
    ? `### 💎 Diamond

<p align="center">
${avatarGridHtml(sponsors.diamond, 120)}
</p>
`
    : ''
}
${
  sponsors.gold.length > 0
    ? `### 🥇 Gold

<p align="center">
${avatarGridHtml(sponsors.gold, 100)}
</p>
`
    : ''
}
${
  sponsors.silver.length > 0
    ? `### 🥈 Silver

<p align="center">
${avatarGridHtml(sponsors.silver, 80)}
</p>
`
    : ''
}
${
  sponsors.backers.length > 0
    ? `### 💜 Backers

<p align="center">
${avatarGridHtml(sponsors.backers, 50)}
</p>
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

${contributorTableHtml(contributors)}

---

## 🐛 Issue Reporters

Những người đã phát hiện lỗi và đề xuất tính năng mới.

<p align="center">
${avatarGridHtml(filteredIssueCreators)}
</p>

---

## 💬 Thảo luận & Góp ý

Những người đã tham gia thảo luận, giúp định hình sản phẩm.

<p align="center">
${avatarGridHtml(filteredCommenters)}
</p>

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

  const [sponsors, contributors, issueCreators, commenters] = await Promise.all(
    [fetchSponsors(), fetchContributors(), fetchIssueCreators(), fetchCommenters()]
  );

  console.log(`Found:`);
  console.log(`  - ${sponsors.diamond.length + sponsors.gold.length + sponsors.silver.length + sponsors.backers.length} sponsors`);
  console.log(`  - ${contributors.length} code contributors`);
  console.log(`  - ${issueCreators.length} issue reporters`);
  console.log(`  - ${commenters.length} commenters`);

  const markdown = generateMarkdown(
    sponsors,
    contributors,
    issueCreators,
    commenters
  );

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
