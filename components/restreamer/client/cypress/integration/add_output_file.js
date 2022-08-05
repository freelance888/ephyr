describe('ADD FILE OUTPUT', () => {
  it('Goes to the homepage', () => {
    cy.visit('/');
  });
  it('Click Output', () => {
    cy.get("span:contains('Output'):first").click();
    cy.wait(5000);
  });

  it('Set optional label', () => {
    cy.get('[placeholder="optional label"]').type('File Record');
  });

  it('Set rtmp://', () => {
    const urlToPaste = 'file:///record.flv';
    cy.get('[placeholder="rtmp://..."]')
      .invoke('val', urlToPaste)
      .trigger('input');
  });

  it('Submits', () => {
    cy.get("button:contains('Add')").click();
    cy.get("button:contains('Add')").should('not.exist');
  });

  it('Assert', () => {
    cy.get("span:contains('File Record')").should('have.text', 'File Record');
    cy.get("a:contains('record.flv'):first").should(
      'have.text',
      'file:///record.flv'
    );
  });
});
