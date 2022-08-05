describe('ALL OUTPUTS START', () => {
  it('Goes to the homepage', () => {
    cy.visit('/');
  });

  it('Click Start All', () => {
    cy.get("span:contains('Start All')").click();
  });

  it('Confirm', () => {
    cy.get('[slot="confirm"]').click();
  });

  it('Assert', () => {
    cy.get(
      ':nth-child(4) > .uk-grid > .uk-first-column > :nth-child(5) > .toggle > input[type="checkbox"]'
    ).should('be.checked');
    cy.get(
      ':nth-child(4) > .uk-grid > :nth-child(2) > :nth-child(5) > .toggle > input[type="checkbox"]'
    ).should('be.checked');
    cy.get(
      ':nth-child(2) > .uk-grid > .uk-first-column > :nth-child(5) > .toggle > input[type="checkbox"]'
    ).should('be.checked');
    cy.get(
      ':nth-child(2) > .uk-grid > :nth-child(2) > :nth-child(5) > .toggle > input[type="checkbox"]'
    ).should('be.checked');
  });
});
