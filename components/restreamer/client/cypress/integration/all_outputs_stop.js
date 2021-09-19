describe('ALL OTPUTS STOP', () => {
  it('Goes to the homepage', () => {
    cy.visit('/');
  });
  it('Click Stop All', () => {
    cy.get("span:contains('Stop All')").click();
  });

  it('Confirm', () => {
    cy.get('[slot="confirm"]').click();
    cy.get('[slot="confirm"]').should('not.exist');
  });

  it('Assert', () => {
    cy.get(
      ':nth-child(4) > .uk-grid > .uk-first-column > :nth-child(5) > .toggle > input[type="checkbox"]'
    ).should('not.be.checked');
    cy.get(
      ':nth-child(4) > .uk-grid > :nth-child(2) > :nth-child(5) > .toggle > input[type="checkbox"]'
    ).should('not.be.checked');
    cy.get(
      ':nth-child(2) > .uk-grid > .uk-first-column > :nth-child(5) > .toggle > input[type="checkbox"]'
    ).should('not.be.checked');
    cy.get(
      ':nth-child(2) > .uk-grid > :nth-child(2) > :nth-child(5) > .toggle > input[type="checkbox"]'
    ).should('not.be.checked');
  });
});
