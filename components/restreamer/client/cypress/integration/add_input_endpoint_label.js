describe.only('CHECK INPUT ENDPOINT LABEL', () => {
  before(() => {
    cy.visit('/');
    cy.deleteAllInputs();
    cy.importJsonConf(Cypress.env('host'));
  });

  after(() => {
    cy.deleteAllInputs();
  });

  it('Assert that first endpoint input does not have label option', () => {
    cy.get("span:contains('/it/playback')")
      .parent()
      .parent()
      .invoke('show')
      .find('.endpoint-label')
      .should('not.exist');
  });

  it('Add label', () => {
    cy.get("span:contains('/it/backup1')")
      .parent()
      .parent()
      .find('.edit-label-btn')
      .invoke('show')
      .click();
    cy.focused().type('Some text{enter}');
  });

  it('Cancel edit label by click Esc', () => {
    cy.get("span:contains('/it/backup1')")
      .parent()
      .parent()
      .find('.edit-label-btn')
      .invoke('show')
      .click();
    cy.focused().type('Text should not be after click Esc{esc}');
  });

  it('Cancel edit label by click outside', () => {
    cy.get("span:contains('/it/backup1')")
      .parent()
      .parent()
      .find('.edit-label-btn')
      .invoke('show')
      .click();
    cy.focused().type('Text should not be after click outside');
    cy.get('html').trigger('mouseover');
  });

  it('Add new backup input should not affect labels', () => {
    cy.get(':nth-child(2) > .edit-input').click();
    cy.get("button:contains('Add backup')").click();
    cy.get('button')
      .contains(/^Edit$/)
      .click();
    cy.get('button')
      .contains(/^Edit$/)
      .should('not.exist');
  });

  it('Assert that endpoint label have text', () => {
    cy.get("span:contains('/it/backup1')")
      .parent()
      .parent()
      .find('[data-testid="endpoint-label-text"]')
      .should('have.text', 'Some text');
  });
});