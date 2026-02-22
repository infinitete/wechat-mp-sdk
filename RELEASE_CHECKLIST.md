# Release Checklist — wechat-mp-sdk v0.1.0

## Endpoint Parity Summary

| Metric | Value |
|--------|-------|
| **Total Endpoints** | 128 |
| **Implemented** | 127 (all non-deprecated) |
| **Deprecated** | 1 (`cloud.sendCloudBaseSms`) |
| **Missing (non-deprecated)** | 0 |
| **Coverage** | 99.21% overall / **100% of non-deprecated** |

## API Categories Covered (24)

| # | Category | Endpoints |
|---|----------|-----------|
| 1 | Login & Auth | 3 — code2Session, checkSessionKey, resetUserSessionKey |
| 2 | Access Token | 2 — getAccessToken, getStableAccessToken |
| 3 | OpenAPI Management | 8 — clearQuota, getApiQuota, clearApiQuota, clearQuotaByAppSecret, getRidInfo, callbackCheck, getApiDomainIp, getCallbackIp |
| 4 | Security | 3 — msgSecCheck, mediaCheckAsync, getUserRiskRank |
| 5 | User Info | 5 — getPhoneNumber, getPluginOpenPId, checkEncryptedData, getPaidUnionid, getUserEncryptKey |
| 6 | QR Code & Links | 9 — getQRCode, getUnlimitedQRCode, createQRCode, generateScheme, queryScheme, generateUrlLink, queryUrlLink, generateShortLink, generateNFCScheme |
| 7 | Customer Service | 4 — sendCustomMessage, uploadTempMedia, getTempMedia, setTyping |
| 8 | Subscribe Messages | 10 — sendMessage, addMessageTemplate, deleteMessageTemplate, getCategory, getMessageTemplateList, getPubTemplateKeyWordsById, getPubTemplateTitleList, setUserNotify, setUserNotifyExt, getUserNotify |
| 9 | Data Analytics | 11 — getDailySummary, getDailyVisitTrend, getWeeklyVisitTrend, getMonthlyVisitTrend, getDailyRetain, getWeeklyRetain, getMonthlyRetain, getVisitPage, getVisitDistribution, getUserPortrait, getPerformanceData |
| 10 | Operations Center | 10 — getDomainInfo, getPerformance, getSceneList, getVersionList, realtimeLogSearch, getFeedback, getFeedbackMedia, getJsErrDetail, getJsErrList, getGrayReleasePlan |
| 11 | Image & OCR | 8 — aiCrop, scanQRCode, printedTextOCR, vehicleLicenseOCR, bankCardOCR, businessLicenseOCR, driverLicenseOCR, idCardOCR |
| 12 | Plugin Management | 2 — managePluginApplication, managePlugin |
| 13 | Nearby Mini Programs | 4 — addNearbyPoi, deleteNearbyPoi, getNearbyPoiList, setShowStatus |
| 14 | Cloud Development | 10 — invokeCloudFunction, addDelayedFunctionTask, databaseAdd, databaseDelete, databaseUpdate, databaseQuery, getUploadFileLink, getDownloadFileLink, deleteCloudFile, newSendCloudBaseSms |
| 15 | Live Streaming | 9 — createRoom, deleteRoom, editRoom, getLiveInfo, addGoods, updateGoodsInfo, deleteGoodsInfo, pushMessage, getFollowers |
| 16 | Hardware & IoT | 6 — sendHardwareDeviceMessage, getSnTicket, createIotGroupId, getIotGroupInfo, addIotGroupDevice, removeIotGroupDevice |
| 17 | Instant Delivery | 5 — getAllImmeDelivery, preAddOrder, preCancelOrder, addLocalOrder, cancelLocalOrder |
| 18 | Express & Logistics | 6 — bindAccount, getAllAccount, getAllDelivery, getOrder, addOrder, getPath |
| 19 | Service Market | 1 — invokeService |
| 20 | Biometric Auth | 1 — verifySignature |
| 21 | Face Verification | 2 — getVerifyId, queryVerifyInfo |
| 22 | WeChat Search | 1 — submitPages |
| 23 | Advertising | 4 — addUserAction, addUserActionSet, getUserActionSetReports, getUserActionSets |
| 24 | WeChat Customer Service | 3 — getKfWorkBound, bindKfWork, unbindKfWork |

## Deprecated Endpoints (Excluded)

| Endpoint ID | Reason |
|-------------|--------|
| `cloud.sendCloudBaseSms` | Officially deprecated by WeChat; replaced by `cloud.newSendCloudBaseSms` |

## Test Suite Summary

| Test Category | Description |
|---------------|-------------|
| **Unit Tests** | Per-module tests for request/response serialization, error handling |
| **HTTP Contract Tests** | Wiremock-based tests verifying HTTP method, path, query params, request/response bodies |
| **Parity Baseline Tests** | Facade vs. direct API module behavioral equivalence |
| **Coverage Matrix Tests** | Endpoint inventory completeness and coverage gate |
| **Facade Guard Tests** | Compile-time and runtime guards ensuring facade stays in sync with inventory |
| **Reliability Tests** | Token management, retry logic, concurrency safety |
| **Doc Tests** | Inline documentation examples |

## Verification Commands

```bash
# 1. Coverage gate test (must pass — asserts missing_non_deprecated == 0)
cargo test coverage_matrix::non_deprecated_missing_endpoints_zero -- --nocapture

# 2. Full coverage matrix with summary output
cargo test coverage_matrix:: -- --nocapture

# 3. Code formatting
cargo fmt --check

# 4. Linting (warnings as errors)
cargo clippy --all-targets --all-features -- -D warnings

# 5. Full test suite
cargo test
```

### Expected Results

- `non_deprecated_missing_endpoints_zero`: **PASS** (0 missing non-deprecated endpoints)
- `coverage_report_prints_summary`: Prints `missing_non_deprecated=0`, `coverage_percent=99.21%`
- `fails_when_endpoint_unmapped`: **PASS** (redundant guard, same assertion)
- `cargo fmt --check`: Clean (no formatting issues)
- `cargo clippy`: Clean (zero warnings)
- `cargo test`: **All tests pass**, 0 failures, 0 ignored

## Pre-Release Verification

- [x] All 127 non-deprecated endpoints implemented across 24 categories
- [x] All facade methods wired in `src/client/wechat_mp.rs`
- [x] Coverage gate test `non_deprecated_missing_endpoints_zero` enabled (no longer `#[ignore]`)
- [x] `cargo fmt --check` passes
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [x] `cargo test` — all tests green
- [x] Documentation builds cleanly (`cargo doc --no-deps`)
- [x] Three working examples in `examples/` directory
