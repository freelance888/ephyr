<script lang="js">
  import UIkit from 'uikit';
  import Icons from 'uikit/dist/js/uikit-icons';
  import { showError } from '../../utils/util';
  import ServerInfo from './ServerInfo.svelte';
  import { Split } from '@geoffcox/svelte-splitter/src/index';
  import Console from '../Console.svelte';

  UIkit.use(Icons);

  export let isLoading;
  export let canRenderToolbar;
  export let canRenderMainComponent;
  export let error;
  export let serverInfo;

  const onToggleConsole = (event) => {
    const primaryPaneHeight = event.detail ? '70%' : '100%';

    let primary = document.querySelector('.split.horizontal');
    primary.style.setProperty('--primary-size', primaryPaneHeight);
  };
</script>

<template>
  <Split horizontal initialPrimarySize="100%" minSecondarySize="40px">
    <div slot="primary" class="page uk-flex uk-flex-column">
      <header class="uk-container">
        <div class="uk-grid uk-grid-small" uk-grid>
          <a
            href="https://creativesociety.com"
            target="_blank"
            rel="noopener noreferrer"
            class="logo uk-flex"
            title="Join us on creativesociety.com"
          >
            <img src="logo.jpg" alt="Logo" />
            <h3>Creative Society</h3>
            <small>Ephyr re-streamer {process.env.VERSION}</small>
          </a>
          {#if !isLoading}
            <ServerInfo {serverInfo} />
          {/if}
          <div class="uk-margin-auto-left">
            {#if canRenderToolbar}
              <slot name="toolbar" />
            {/if}
            {#if error}
              {showError(error.message) || ''}
            {/if}
          </div>
        </div>
      </header>

      <main class="uk-container uk-flex-1">
        {#if isLoading}
          <div class="uk-alert uk-alert-warning loading">Loading...</div>
        {:else if canRenderMainComponent}
          <slot name="main" />
        {/if}
      </main>

      <footer class="uk-container">
        Developed for people with ❤ by
        <a href="https://github.com/ALLATRA-IT" target="_blank noreferrer"
        >AllatRa IT</a
        >
      </footer>
    </div>
    <Console slot="secondary" on:toggleConsole={onToggleConsole} />
  </Split>
</template>

<style lang="stylus" global>
  @require "../../../node_modules/uikit/dist/css/uikit.min.css"
  :root {
    --primary-text-color: #777
    --primary-text-hover-color: #333
    --warning-color: #faa05a
    --success-color: #32d296
    --danger-color: #f0506e
    --secondary-warning-color: #7a5128
  }

  html,
  body
    height: 100%

  .circle
    background-color: currentColor
    border-radius: 50%
    width: 6px
    height 6px
    display: inline-block
    margin-bottom: 2px

  .initializing
    color: var(--warning-color)

  .online
    color: var(--success-color)

  .offline
    color: var(--danger-color)

  .unstable
    color: var(--secondary-warning-color)

  .page
    min-height: calc(100vh - 60px);

  h2, h3
    color: var(--primary-text-color)

  .uk-container
    padding-left: 34px !important
    padding-right: @padding-left
    max-width: auto !important
    width: calc(100% - 68px)
    min-width: 320px

  .split.horizontal
    max-height: 100vh !important;

  .primary
    overflow-y: auto !important;

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

  .drop-target, #dnd-action-dragged-el
    outline: none !important;

  footer
    padding-top: 10px
    padding-bottom: 3px
    font-size: 12px

  .uk-flex-base-line
    align-items: baseline

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

  .toolbar
    position: relative
    padding: 8px
    margin-top: 20px
    margin-bottom: 30px

  .toolbar-label
    line-height: 38px
    margin-left: 12px

  .section-label
    font-size: smaller
    font-weight: bold
    position: absolute
    top: -16px
    left: 0
    padding: 2px 10px
    border-top-left-radius: 4px
    border-top-right-radius: 4px
    background-color: #f8f8f8

  .uk-alert
    font-size: 14px
    margin: 5px 0 10px 0

  .uk-checkbox
    margin: 10px 0 10px 0

  .hidden
    display: none

  .uk-tooltip
    max-width: 400px

  .has-error
    color: var(--danger-color)

  .has-warning
    color: var(--warning-color)

  .url-action-btn
    white-space: nowrap
    padding-right: 1em
    padding-left: 1em
    align-self: center
    height: 100%
    color: var(--primary-text-color)
    opacity: 0
    text-transform: initial
    text-decoration: none
    font-size: 13px
    transition: 0.1s ease-in

    &:hover
      color: var(--primary-text-hover-color)
</style>
