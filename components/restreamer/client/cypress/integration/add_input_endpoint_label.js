describe('CHECK INPUT ENDPOINT LABEL', () => {
  before(() => {
    cy.visit('/');
    cy.dumpState();
    cy.deleteAllInputs();
    cy.importJsonConf(Cypress.env('host'));
  });

  after(() => {
    cy.restoreState();
  });

  it('Assert that first endpoint input does not have label option', () => {
    cy.get("span:contains('/it/playback')")
      .parent()
      .parent()
      .find('.endpoint-label')
      .should('not.exist');
  });

  it('Add label', () => {
    cy.get("span:contains('/it/backup1')")
      .parent()
      .parent()
      .find('.edit-label')
      .click();
    cy.focused().type('Some text{enter}');
  });

  it('Cancel edit label by click Esc', () => {
    cy.get("span:contains('/it/backup1')")
      .parent()
      .parent()
      .find('.edit-label')
      .click();
    cy.focused().type('Text should not be after click Esc{esc}');
  });

  it('Cancel edit label by click outside', () => {
    cy.get("span:contains('/it/backup1')")
      .parent()
      .parent()
      .find('.edit-label')
      .click();
    cy.focused().type('Text should not be after click outside');
    cy.get('html').trigger('mouseover');
  });

  it('Assert that endpoint label have text', () => {
    cy.get("span:contains('/it/backup1')")
      .parent()
      .parent()
      .find('.endpoint-label span')
      .should('have.text', 'Some text');
  });
});
