module.exports = async ({ github, context }) => {
    const body = process.env.COMMENT_BODY;

    // Find existing comment from this workflow
    const { data: comments } = await github.rest.issues.listComments({
        owner: context.repo.owner,
        repo: context.repo.repo,
        issue_number: context.issue.number,
    });

    const botComment = comments.find(comment =>
        comment.user.type === 'Bot' &&
        comment.body.includes('Test Results')
    );

    if (botComment) {
        await github.rest.issues.updateComment({
            owner: context.repo.owner,
            repo: context.repo.repo,
            comment_id: botComment.id,
            body: body
        });
    } else {
        await github.rest.issues.createComment({
            owner: context.repo.owner,
            repo: context.repo.repo,
            issue_number: context.issue.number,
            body: body
        });
    }
}
