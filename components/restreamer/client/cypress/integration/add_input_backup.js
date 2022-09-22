describe('ADD BACKUP INPUT', () => {
  it('Goes to the homepage', () => {
    cy.visit('/');
  });
  it('Add-input', () => {
    cy.get('.add-input').click();
  });

  it('Set optional label', () => {
    cy.get('[placeholder="optional label"]').type('BACKUP');
  });

  it('Set stream-key', () => {
    cy.get('[placeholder="<stream-key>"]').type('it');
  });

  it('Checks a checkbox', () => {
    cy.get("button:contains('Add backup')").click();
  });

  it('Submits', () => {
    cy.get('button').contains(/^Add$/).click();
    cy.get('button').contains(/^Add$/).should('not.exist');
  });

  it('Assert', () => {
    cy.get("span:contains('/it/primary')").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/it/primary'
    );
    cy.get("span:contains('/it/backup1')").should(
      'have.text',
      'rtmp://' + Cypress.env('host') + '/it/backup1'
    );
  });
});
