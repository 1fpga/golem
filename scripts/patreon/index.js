import {createPromptModule} from "inquirer";
import timesago from "timesago";
import {simpleGit} from 'simple-git';

const inquirer = {prompt: createPromptModule({output: process.stderr})};

class PatreonClient {
    constructor(api_token) {
        this.token = api_token;
    }

    async fetchUrl(url) {
        const RESPONSE = await fetch(url, {
            headers: {
                Authorization: `Bearer ${this.token}`,
                'User-Agent': 'Patreon API Client',
            },
        });

        if (RESPONSE.status !== 200) {
            throw new Error(`Failed to fetch ${url}: ${RESPONSE.status} ${RESPONSE.statusText}`);
        }
        return await RESPONSE.json();
    }

    async fetchJson(endpoint) {
        return await this.fetchUrl(`https://www.patreon.com/api/oauth2/api/${endpoint}`);
    }

    async fetchCurrentUser() {
        return (await this.fetchJson('current_user')).data;
    }

    async fetchCampaigns() {
        return (await this.fetchJson('current_user/campaigns')).data;
    }

    async fetchPledges(campaign_id) {
        return (await this.fetchJson(`campaigns/${campaign_id}/pledges`)).data;
    }

    async fetchUser(user_id) {
        return (await this.fetchUrl(`https://www.patreon.com/api/user/${user_id}`)).data;
    }

    async fetchReward(reward_id) {
        return (await this.fetchUrl(`https://www.patreon.com/api/rewards/${reward_id}`)).data;
    }
}

async function main() {
    const git = simpleGit();

    const API_TOKEN = process.env['PATREON_API_TOKEN'] ?? (
        (await inquirer.prompt({
            name: "API_TOKEN",
            message: "Enter your Patreon API token:",
            type: "password",
        }, {
            output: process.stderr,
        })).API_TOKEN
    );
    let LAST_RELEASE = process.env['LAST_RELEASE'] ?? (
        (await inquirer.prompt({
            name: "LAST_RELEASE",
            message: "Enter the last release tag:",
            type: "input",
        }, {
            output: process.stderr,
        })).LAST_RELEASE
    );
    if (!LAST_RELEASE) {
        LAST_RELEASE = (await git.tags()).latest;
    }

    let client = new PatreonClient(API_TOKEN);
    let user = await client.fetchCurrentUser();
    let full_name = user.attributes.full_name;

    console.error(`Hello, ${full_name}!`);
    console.error("Fetching your campaigns...");
    let campaigns = await client.fetchCampaigns();
    console.error(`Found ${campaigns.length} campaigns.`);

    let campaign_id;
    if (campaigns.length === 1) {
        campaign_id = campaigns[0].id;
    } else {
        campaign_id = (await inquirer.prompt({
            name: "CAMPAIGN_ID",
            message: "Enter the campaign ID:",
            type: "input",
            choices: campaigns.map(campaign => ({
                name: campaign.attributes.creation_name,
                value: campaign.id,
            })),
        }, {
            output: process.stderr,
        })).CAMPAIGN_ID;
    }

    console.error(`Fetching the campaign ${campaign_id}...`);
    const PLEDGES = await client.fetchPledges(campaign_id);
    console.error(`Found ${PLEDGES.length} pledges.`);

    const tiers = {
        0: "Contributors",
    };
    const patrons = {};

    // The list of names to be replaced by aliases.
    const replace_patrons_names = (await import("./replace_patrons_names.json", {with: {type: "json"}})).default;

    for (let pledge of PLEDGES) {
        let user = pledge.relationships.patron.data;
        let reward = pledge.relationships.reward.data;
        let since = timesago(new Date(pledge.attributes.created_at), {
            suffixAgo: "",
        });
        let [user_data, reward_data] = await Promise.all([
            client.fetchUser(user.id),
            client.fetchReward(reward.id)
        ]);
        let user_name = user_data.attributes.full_name;
        if (replace_patrons_names[user_name]) {
            user_name = replace_patrons_names[user_name];
        }

        let reward_tier = reward_data.attributes.title;
        tiers[reward_data.attributes.amount] = reward_tier;

        patrons[reward_tier] = patrons[reward_tier] || [];
        patrons[reward_tier].push(`${user_name.trim()} (${since})`);
    }

    // Contributors.
    const contributors = new Set();
    let logs = await git.log({
        from: LAST_RELEASE,
    });
    for (let log of logs.all) {
        let author = log.author_name;
        if (replace_patrons_names[author]) {
            author = replace_patrons_names[author];
        }
        contributors.add(author);
    }
    patrons[tiers[0]] = Array.from(contributors);

    console.log(JSON.stringify({tiers, patrons}, null, 4));
}

main().then(() => {
    console.error("Done.");
    process.exit(0);
}, (e) => {
    console.error("An error occurred:");
    console.error(e);
    process.exit(1);
});
