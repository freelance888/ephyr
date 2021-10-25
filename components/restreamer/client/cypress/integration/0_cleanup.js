describe('REMOVE ALL INPUTS', () => {
  it('imports empty array', () => {
    cy.visit('/');
    cy.get('[title="Export/Import all"]').click({ force: true });
    const textArea = '[placeholder="JSON..."]';
    cy.get(textArea).clear();
    cy.get(textArea).type(`{
      "restreams":[],
      "version": "v1"
    }`);
    cy.get("button:contains('Replace')").click();
  });
});
