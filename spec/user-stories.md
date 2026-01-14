# Filmorator User Stories

Written using User Story Chef protocol. Stories are value negotiation units. AC are falsifiable hypotheses.

---

## Epic 1: Campaign Creation

Owner transforms photos into a rankable campaign.

### Story 1.1: Create Empty Campaign

**Outcome:** Owner has a campaign entity they can populate with photos.

```
Given I have photos I want ranked
When I create a campaign with a name
Then I receive a management URL I can use to control it
And losing this URL means losing the campaign (no recovery)
```

**AC (Hypotheses):**
- Owner can create campaign in under 30 seconds
- Management URL is the only way to access owner functions
- Campaign name appears in participant UI

---

### Story 1.2: Add Photos to Campaign

**Outcome:** Owner's JPEG files become rankable photos in the campaign.

```
Given I have an empty campaign
When I add a folder of JPEGs
Then each valid JPEG becomes a photo in the campaign
And invalid files are rejected with clear error messages
And the campaign becomes immutable once photos are added
```

**AC (Hypotheses):**
- 50-200 photos upload without timeout
- Non-JPEG files rejected with filename + reason
- Photos display in original aspect ratio (no crops)
- After photos added, no modification possible

---

### Story 1.3: Generate Shareable Link

**Outcome:** Owner has a URL they can share with participants.

```
Given I have a campaign with photos
When I request the participant link
Then I receive a URL that anyone can use to contribute rankings
And the URL works without any login or authentication
```

**AC (Hypotheses):**
- URL is short enough to share in messages
- Opening URL on mobile shows comparison UI immediately
- No login prompt, no cookie consent, no friction

---

## Epic 2: Photo Comparison

Participant contributes rankings to a campaign.

### Story 2.1: View Comparison

**Outcome:** Participant sees 3 photos and understands what to do.

```
Given I opened a campaign link
When the page loads
Then I see 3 photos and clear instruction to rank them
And I understand the task in under 10 seconds
```

**AC (Hypotheses):**
- Photos load fast (thumbnails first, then previews)
- Instruction text is unambiguous
- Works on mobile without horizontal scrolling
- No confusion about what "rank" means

---

### Story 2.2: Submit Ranking

**Outcome:** Participant's preference is recorded and they can continue.

```
Given I see 3 photos
When I tap them in preference order (best first)
Then my ranking is recorded
And I immediately see the next 3 photos
And I can see my contribution count increasing
```

**AC (Hypotheses):**
- Ranking a set takes under 10 seconds
- Undo is possible before final submission
- Contribution count updates immediately
- No page reload between comparisons

---

### Story 2.3: Undo Selection

**Outcome:** Participant can correct mistakes without starting over.

```
Given I've tapped one or two photos
When I tap an already-selected photo
Then my selection is removed
And I can re-select in correct order
```

**AC (Hypotheses):**
- Tap toggles selection (not just add)
- Visual feedback shows current selection state
- Can completely restart selection without submitting

---

### Story 2.4: Identify Myself (Optional)

**Outcome:** Participant can attach their name to contributions.

```
Given I'm contributing rankings
When I enter my name in the optional field
Then my name is associated with my session
And I can change it anytime
And the owner will see my name with my contribution count
```

**AC (Hypotheses):**
- Name field is visible but not required
- No validation beyond reasonable length
- Change propagates to existing contributions
- Empty name = "Anonymous" in owner view

---

## Epic 3: Progress & Feedback

Participant sees their impact and campaign status.

### Story 3.1: See Contribution Count

**Outcome:** Participant knows how much they've contributed.

```
Given I've submitted rankings
When I look at the UI
Then I see "You've ranked X matchups"
And the count is always current
```

**AC (Hypotheses):**
- Count visible without navigating away from comparison UI
- Updates immediately after each submission
- Persists across browser sessions (same device)

---

### Story 3.2: See Campaign Progress

**Outcome:** Participant knows how complete the campaign is.

```
Given the campaign has received contributions
When I look at the UI
Then I see campaign completion percentage
And I understand my contribution to that progress
```

**AC (Hypotheses):**
- Progress is meaningful (not arbitrary)
- Shows "X% complete" or similar
- Progress can exceed 100% if collection continues past threshold

---

### Story 3.3: See My Impact

**Outcome:** Participant knows their rankings affected the outcome.

```
Given I've submitted a ranking
When I look at feedback
Then I see how my comparison affected photo positions
```

**AC (Hypotheses):**
- Shows "Photo X moved up/down N positions"
- Feedback appears after submission, not blocking next comparison
- Impact is real, not fabricated

---

### Story 3.4: See Other Contributors

**Outcome:** Participant knows they're part of a group effort.

```
Given others have contributed
When I look at the UI
Then I see who else has contributed (names if provided)
And I feel part of a collective effort
```

**AC (Hypotheses):**
- Shows "Alice, Bob, and N others contributed"
- Only shows names of those who provided them
- Creates social proof without requiring names

---

## Epic 4: Results Access

Participant sees ranking outcomes.

### Story 4.1: Unlock Results Preview

**Outcome:** Participant earns access to current rankings through contribution.

```
Given I haven't contributed enough
When I try to view results
Then I see how many more comparisons needed to unlock
And I'm motivated to continue
```

**AC (Hypotheses):**
- Unlock threshold based on statistical validity, not arbitrary count
- Clear progress toward unlock
- Results actually unlock after threshold met

---

### Story 4.2: View Current Rankings

**Outcome:** Participant sees the aggregated ranking.

```
Given I've unlocked results
When I view the ranking page
Then I see all photos ordered by aggregate preference
And I can see relative strength/confidence
```

**AC (Hypotheses):**
- Ranking order matches Bradley-Terry output
- Visual indication of confidence/uncertainty
- Photos maintain aspect ratio in ranking view

---

### Story 4.3: Compare My Rankings to Aggregate

**Outcome:** Participant sees how their taste differs from the crowd.

```
Given I've contributed and unlocked results
When I view the comparison
Then I see where my rankings differed from aggregate
And I learn something about my preferences
```

**AC (Hypotheses):**
- Shows "You ranked Photo X higher/lower than crowd"
- Interesting, not judgmental
- Based on actual submitted rankings

---

## Epic 5: Campaign Management

Owner monitors and closes campaigns.

### Story 5.1: View Contributor List

**Outcome:** Owner knows who helped and how much.

```
Given my campaign has received contributions
When I visit the management URL
Then I see each contributor with their comparison count
And I can thank them appropriately
```

**AC (Hypotheses):**
- Shows names where provided, "Anonymous" otherwise
- Shows comparison count per contributor
- Sorted by contribution amount

---

### Story 5.2: Receive Threshold Notification

**Outcome:** Owner knows when enough data exists for valid ranking.

```
Given contributions are accumulating
When statistical threshold is reached
Then I'm notified that ranking is now meaningful
And I can choose to close or continue collecting
```

**AC (Hypotheses):**
- Notification reaches owner (mechanism TBD)
- Explains what threshold means
- Campaign continues collecting if not closed

---

### Story 5.3: Close Campaign

**Outcome:** Owner finalizes the campaign.

```
Given my campaign is collecting
When I close it
Then no new comparisons are accepted
And final results are locked
And participants can view final ranking
```

**AC (Hypotheses):**
- Closing is irreversible (until reopen)
- Clear confirmation before close
- Final ranking visible to all

---

### Story 5.4: Reopen Campaign

**Outcome:** Owner can gather more data if needed.

```
Given my campaign is closed
When I reopen it
Then comparisons can be submitted again
And ranking updates with new data
```

**AC (Hypotheses):**
- Reopen preserves existing data
- Clear indication campaign is active again
- Can close again when satisfied

---

### Story 5.5: View Suspicious Activity

**Outcome:** Owner can assess potential abuse.

```
Given someone submitted suspicious patterns
When I view the contributor list
Then suspicious sessions are flagged
And I can exclude them from ranking calculation
```

**AC (Hypotheses):**
- Suspicious = patterns inconsistent with genuine preference
- Flag is visible but not automatic exclusion
- Owner makes final decision
- Excluded data doesn't affect ranking

---

## Story Dependencies

```
1.1 Create Campaign
 ↓
1.2 Add Photos
 ↓
1.3 Generate Link
 ↓
2.1 View Comparison → 2.2 Submit → 2.3 Undo (parallel)
                              ↓
                    2.4 Identify (optional, parallel)
                              ↓
3.1 Count → 3.2 Progress → 3.3 Impact → 3.4 Contributors
                              ↓
                    4.1 Unlock → 4.2 Rankings → 4.3 Compare
                              ↓
5.1 Contributors → 5.2 Threshold → 5.3 Close → 5.4 Reopen
                              ↓
                         5.5 Suspicious
```

---

## Not In MVP (Explicitly Deferred)

- QR code generation
- Embed widget
- Gamification (points, streaks)
- Account system
- Photo context/metadata
- Multiple image formats
