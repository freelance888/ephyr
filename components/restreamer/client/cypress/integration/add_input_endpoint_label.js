describe('CHECK INPUT ENDPOINT LABEL', () => {
  it('Goes to the homepage', () => {
    cy.visit('/');
  });

  it('Add input with backup', () => {
    cy.get('.add-input').click();
    cy.get('[placeholder="optional label"]').type('CHECK INPUT ENDPOINT LABEL');
    cy.get('[placeholder="<stream-key>"]').type('check-label');
    cy.get("button:contains('Add backup')").click();
    cy.get('button').contains(/^Add$/).click();
    cy.get('button').contains(/^Add$/).should('not.exist');
  });

  it('Assert that first endpoint input does not have label option', () => {
    cy.get("span:contains('/check-label/playback')")
      .parent()
      .parent()
      .find('.endpoint-label')
      .should('not.exist');
  });

  it('Add label', () => {
    cy.get("span:contains('/check-label/backup1')")
      .parent()
      .parent()
      .find('.edit-label')
      .click();
    cy.focused().type('Some text{enter}');
  });

  it('Cancel edit label by click Esc', () => {
    cy.get("span:contains('/check-label/backup1')")
      .parent()
      .parent()
      .find('.edit-label')
      .click();
    cy.focused().type('Text should not be after click Esc{esc}');
  });

  it('Cancel edit label by click outside', () => {
    cy.get("span:contains('/check-label/backup1')")
      .parent()
      .parent()
      .find('.edit-label')
      .click();
    cy.focused().type('Text should not be after click outside');
    cy.get('html').trigger('mouseover');
  });

  it('Assert that endpoint label have text', () => {
    cy.get("span:contains('/check-label/backup1')")
      .parent()
      .parent()
      .find('.endpoint-label span')
      .should('have.text', 'Some text');
  });
});
