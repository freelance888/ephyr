<script lang="js">
  import UIkit from 'uikit';
  import Icons from 'uikit/dist/js/uikit-icons';
  import { showError } from './util';
  UIkit.use(Icons);

  export let isStateLoading;
  export let canRenderToolbar;
  export let canRenderMainComponent;
  export let error;
</script>

<template>
  <div class="page uk-flex uk-flex-column">
    <header class="uk-container">
      <div class="uk-grid uk-grid-small" uk-grid>
        <a
          href="https://allatraunites.com"
          target="_blank"
          class="logo uk-flex"
          title="Join us on allatraunites.com"
        >
          <img src="logo.jpg" alt="Logo" />
          <h3>Creative Society</h3>
          <small>Ephyr re-streamer {process.env.VERSION}</small>
        </a>
        <div class="uk-margin-auto-left">
          {#if canRenderToolbar}
            <slot name="toolbar" />
          {/if}
          {#if error}
            {showError(error.message)}
          {/if}
        </div>
      </div>
    </header>

    <main class="uk-container uk-flex-1">
      {#if isStateLoading}
        <div class="uk-alert uk-alert-warning loading">Loading...</div>
      {:else if canRenderMainComponent}
        <slot name="main" />
      {/if}
    </main>

    <footer class="uk-container">
      Developed for people with ❤ by
      <a href="https://github.com/ALLATRA-IT" target="_blank">AllatRa IT</a>
    </footer>
  </div>
</template>

<style lang="stylus" global>
  @require "../node_modules/uikit/dist/css/uikit.min.css"
  :root {
    --primary-text-color: #777;
  }

  .page
    min-height: 100vh;

  h2, h3
    color: var(--primary-text-color)

  .uk-container
    padding-left: 34px !important
    padding-right: @padding-left
    max-width: auto !important
    width: calc(100% - 68px)
    min-width: 320px

  header
    padding: 10px

    .logo
      outline: none
      position: relative
      white-space: nowrap
      &:hover
        text-decoration: none

      img
        width: 44px
        height: @width

      h3
        margin: 4px 4px 4px 8px
        max-width: 50%

      small
        position: absolute
        font-size: 12px
        bottom: -6px
        left: 83px
        color: #999

  main
    > .loading
      text-align: center

  .uk-button-primary
    background-color: #08c
    &:not([disabled]):hover
      background-color: #046

  footer
    padding-top: 10px
    padding-bottom: 3px
    font-size: 12px

  .uk-notification-message
    pointer-events: none
    font-size: 1rem
    overflow-wrap: anywhere
    & > div
      padding-right: 14px

    .uk-notification-close
      display: inherit
      pointer-events: all

    .uk-icon-link
      pointer-events: all

  .overflow-wrap
    overflow-wrap: anywhere;
    white-space: normal;

</style>
